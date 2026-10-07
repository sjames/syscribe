// Orchestrates the sprotty editor for one persistent host div, across
// however many diagram tabs `base.html`'s tab bar opens (`ADR-SYS-DE-001`,
// `REQ-TRS-DE-004`). One `DiagramEditor` instance, one DI container, one
// `LocalModelSource` — switching tabs calls `setModel(...)` with a different
// cached schema rather than tearing anything down, since sprotty's
// `ModelViewer` patches a *specific* DOM node by id (`viewerOptions.baseDiv`)
// and losing that node (e.g. via an `innerHTML` reset elsewhere) would break
// future patches — see the code comment on `HOST_ID` below.
//
// The schema is the nested graph of `REQ-TRS-VIS-006` (see `types.ts`), so
// every lookup walks the tree (`findShape`/`containerOf`/`removeFromTree`)
// rather than scanning `model.children`, and every position — in the schema,
// in `MoveAction`, and in what `PATCH /api/diagrams/layout` receives — is
// relative to the element's parent (sprotty's own convention; the server's
// pin semantics match, see `vis::sprotty`'s module doc).
//
// Layout (`REQ-TRS-VIS-007`): `setModel` runs sprotty's hidden measuring pass
// and then ELK (`container.ts`/`layout.ts`); the result lives only in the
// cached schema. The model changes only when the user drags a node (one
// pin), presses *Pin all* (every shape, one PATCH) or *Auto-layout* (DELETE
// every pin, re-fetch, re-run ELK).
import 'reflect-metadata';
import { Container } from 'inversify';
import { IActionDispatcher, LocalModelSource, MouseTool, MoveMouseListener, SelectMouseListener, TYPES } from 'sprotty';
import {
    CreateElementAction,
    DeleteElementAction,
    ElementMove,
    FitToScreenAction,
    MoveAction,
    SetViewportAction,
} from 'sprotty-protocol';
import * as api from './api';
import { isDerivedDiagram, portChain, resolveConnectEnds } from './connect-rules';
import { createDiagramContainer } from './container';
import { ConnectMouseListener } from './connect-listener';
import { defaultSize, prepareForLayout, prepareNode } from './layout';
import { serialiseDiagramSvg } from './svg-export';
import {
    allShapes,
    containerOf,
    DiagramModelSchema,
    findShape,
    Finding,
    isEdgeSchema,
    removeFromTree,
    subtreeIds,
    SysmlChildSchema,
    SysmlEdgeSchema,
    SysmlNodeSchema,
    WriteResponse,
} from './types';

/** Id of the persistent DOM div sprotty renders into — see `index.html`. It
 * must never be recreated (no `innerHTML = ...` on it or an ancestor) for as
 * long as this editor instance lives, or sprotty's snabbdom patcher loses its
 * reference to the live DOM node and stops updating anything. */
const HOST_ID = 'sprotty-host';

function summarizeFindings(findings: Finding[]): string {
    return findings.map(f => `${f.code}: ${f.message}`).join('; ');
}

function refusalText(resp: WriteResponse): string {
    return resp.reason ?? summarizeFindings(resp.newErrors);
}

function nodeName(qname: string): string {
    const parts = qname.split('::');
    return parts[parts.length - 1] || qname;
}

export class DiagramEditor {
    private readonly container: Container;
    private readonly dispatcher: IActionDispatcher;
    private readonly modelSource: LocalModelSource;
    private readonly mouseTool: MouseTool;
    private readonly moveListener: MoveMouseListener;
    private readonly selectListener: SelectMouseListener;
    private readonly connectListener = new ConnectMouseListener();

    private readonly cache = new Map<string, DiagramModelSchema>();
    private currentQname: string | null = null;
    private readonly selectedIds = new Set<string>();
    private connectMode = false;

    constructor() {
        this.container = createDiagramContainer(HOST_ID, {
            onMoveFinished: moves => this.handleMoveFinished(moves),
            onSelectionChanged: (sel, desel) => this.handleSelectionChanged(sel, desel),
        });
        this.dispatcher = this.container.get<IActionDispatcher>(TYPES.IActionDispatcher);
        this.modelSource = this.container.get(LocalModelSource);
        this.mouseTool = this.container.get(MouseTool);
        this.moveListener = this.container.get(MoveMouseListener);
        this.selectListener = this.container.get(SelectMouseListener);
        this.connectListener.onConnected = (source, target) => {
            void this.handleConnect(source, target);
        };
    }

    /** Whether `qname` has a cached (possibly locally-edited) model already —
     * lets `base.html` decide whether opening a tab needs a network fetch. */
    isCached(qname: string): boolean {
        return this.cache.has(qname);
    }

    /** Load (if not cached) and mount `qname` as the active diagram. */
    async activate(qname: string): Promise<void> {
        this.currentQname = qname;
        this.selectedIds.clear();
        this.exitConnectModeIfActive();
        let model = this.cache.get(qname);
        if (!model) {
            model = await api.fetchDiagramModel(qname);
            // Micro-layout for the measuring pass; ELK runs inside `setModel`.
            prepareForLayout(model);
            this.cache.set(qname, model);
        }
        await this.modelSource.setModel(model);
    }

    /** Drop a diagram's cached (in-memory, possibly edited) model — called
     * when its tab is closed, so reopening it re-fetches a clean copy. */
    forget(qname: string): void {
        this.cache.delete(qname);
        if (this.currentQname === qname) {
            this.currentQname = null;
        }
    }

    // -----------------------------------------------------------------
    // Create node (REQ-TRS-DE-004)
    // -----------------------------------------------------------------

    async addNode(): Promise<void> {
        const qname = this.currentQname;
        const model = this.activeModel();
        if (!qname || !model) {
            return;
        }
        const ref = window.prompt('New element qualified name (e.g. UAV::NewPart):');
        if (!ref) {
            return;
        }
        const kind = window.prompt('Element type (e.g. PartDef, Requirement, TestCase):', 'PartDef');
        if (!kind) {
            return;
        }
        const shapeId = `s-${ref.replace(/[^A-Za-z0-9]+/g, '-').toLowerCase()}-${Date.now().toString(36)}`;
        const position = this.nextCascadePosition(model);
        const name = nodeName(ref);
        // A new shape is a root-level block (the manifest's `kind:` is the
        // element type, which the IR maps to the `block` role); its label
        // child mirrors what `vis::sprotty` would emit on the next fetch,
        // and `prepareNode` gives it the same micro-layout as a fetched one.
        const schema: SysmlNodeSchema = {
            id: shapeId,
            type: 'node',
            ref,
            resolved: true,
            kind: 'block',
            elementType: kind,
            name,
            position,
            size: defaultSize('block'),
            children: [{ id: `${shapeId}-label`, type: 'label', text: name, position: { x: 8, y: 8 } }],
        };
        prepareNode(schema);
        // The placeholder size is not a server size: let the next measuring
        // pass size the new block from its label.
        delete schema.serverSize;
        schema.layoutOptions = { ...schema.layoutOptions, resizeContainer: true };

        // Optimistic apply.
        model.children.push(schema);
        await this.dispatcher.dispatch(CreateElementAction.create(schema, { containerId: model.id }));

        const resp = await api.createElement({
            qname: ref,
            type: kind,
            diagram: { qname, shapeId, x: position.x, y: position.y, kind },
        });

        if (!resp.written) {
            removeFromTree(model, [shapeId]);
            await this.dispatcher.dispatch(DeleteElementAction.create([shapeId]));
            this.toast(`Create failed: ${refusalText(resp)}`);
        }
    }

    // -----------------------------------------------------------------
    // Delete node (REQ-TRS-DE-004/005)
    // -----------------------------------------------------------------

    async deleteSelected(): Promise<void> {
        const model = this.activeModel();
        if (!model || this.selectedIds.size === 0) {
            return;
        }
        const ids = [...this.selectedIds];
        this.selectedIds.clear();
        for (const id of ids) {
            await this.deleteNode(model, id);
        }
    }

    private async deleteNode(model: DiagramModelSchema, shapeId: string): Promise<void> {
        const node = findShape(model, shapeId);
        if (!node) {
            return;
        }
        // The node's whole subtree goes with it (ports, labels, nested blocks),
        // and so does every root edge touching anything in that subtree. The
        // server prunes the same set from the manifest on its side.
        const removedNodeIds = subtreeIds(node);
        const connectedEdges = model.children
            .filter(isEdgeSchema)
            .filter(e => removedNodeIds.includes(e.sourceId) || removedNodeIds.includes(e.targetId));
        const parent = containerOf(model, shapeId);
        const parentId = parent ? parent.id : model.id;
        const removedIds = [shapeId, ...connectedEdges.map(e => e.id)];

        // Optimistic apply (sprotty's delete removes the subtree with the node).
        removeFromTree(model, removedIds);
        await this.dispatcher.dispatch(DeleteElementAction.create(removedIds));

        const resp = await api.deleteElement(node.ref);
        if (resp.written) {
            return;
        }

        // Revert: disk is unchanged (REQ-TRS-DE-005), so put the node back
        // into its original container and the edges back at the root, exactly
        // as they were.
        const parentChildren = (parent as { children?: SysmlChildSchema[] } | undefined)?.children;
        if (parentChildren && parent !== model) {
            parentChildren.push(node);
        } else {
            model.children.push(node);
        }
        model.children.push(...connectedEdges);
        await this.dispatcher.dispatch(CreateElementAction.create(node, { containerId: parentId }));
        for (const edge of connectedEdges) {
            await this.dispatcher.dispatch(CreateElementAction.create(edge, { containerId: model.id }));
        }
        if (resp.blockedBy && resp.blockedBy.length > 0) {
            const refs = resp.blockedBy.map(b => b.qname).join(', ');
            this.toast(`Delete blocked — still referenced by: ${refs}`);
        } else {
            this.toast(`Delete failed: ${refusalText(resp)}`);
        }
    }

    // -----------------------------------------------------------------
    // Connect edge (REQ-TRS-DE-004, port-aware per REQ-TRS-VIS-008)
    // -----------------------------------------------------------------

    toggleConnectMode(): boolean {
        this.connectMode = !this.connectMode;
        if (this.connectMode) {
            this.mouseTool.deregister(this.moveListener);
            this.mouseTool.deregister(this.selectListener);
            this.mouseTool.register(this.connectListener);
        } else {
            this.exitConnectModeIfActive();
        }
        return this.connectMode;
    }

    private exitConnectModeIfActive(): void {
        if (!this.connectMode) {
            return;
        }
        this.connectMode = false;
        this.connectListener.reset();
        this.mouseTool.deregister(this.connectListener);
        this.mouseTool.register(this.moveListener);
        this.mouseTool.register(this.selectListener);
    }

    /** The gesture starts and ends on ports. A block stands in for its one
     * compatible port; otherwise the gesture is refused with a toast
     * (`connect-rules.ts`). The edge joins the two port ids at the root. */
    private async handleConnect(sourceShapeId: string, targetShapeId: string): Promise<void> {
        const qname = this.currentQname;
        const model = this.activeModel();
        if (!qname || !model) {
            return;
        }
        const sourceShape = findShape(model, sourceShapeId);
        const targetShape = findShape(model, targetShapeId);
        if (!sourceShape || !targetShape) {
            return;
        }
        const ends = resolveConnectEnds(sourceShape, targetShape);
        if (!ends.ok) {
            this.toast(`Connect refused: ${ends.reason}`);
            return;
        }
        const { source, target } = ends;

        const edgeId = `e-${source.id}-${target.id}-${Date.now().toString(36)}`;
        const schema: SysmlEdgeSchema = {
            id: edgeId,
            type: 'edge',
            sourceId: source.id,
            targetId: target.id,
            kind: 'connection',
        };

        // Optimistic apply.
        model.children.push(schema);
        await this.dispatcher.dispatch(CreateElementAction.create(schema, { containerId: model.id }));

        // The diagram's `subject:` is the natural "owning element" for a
        // connect gesture's `connections:` mutation (see
        // `routes::diagram_model`'s doc comment) — the diagram itself has no
        // `connections:` list of its own. Falls back to the diagram's own
        // qname if `subject` is unset. `from`/`to` are the dotted chains
        // `add_connection` resolves relative to that owner. A derived diagram
        // has no manifest to sync an edge into, so the `diagram:` block is
        // sent only for a manifest diagram.
        const ownerQname = model.subject ?? qname;
        const derived = isDerivedDiagram(model);
        const resp = await api.addConnection({
            qname: ownerQname,
            from: portChain(model, source, ownerQname),
            to: portChain(model, target, ownerQname),
            diagram: derived ? undefined : { qname, edgeId, sourceShapeId: source.id, targetShapeId: target.id },
        });

        if (!resp.written) {
            removeFromTree(model, [edgeId]);
            await this.dispatcher.dispatch(DeleteElementAction.create([edgeId]));
            this.toast(`Connect failed: ${refusalText(resp)}`);
        }
    }

    // -----------------------------------------------------------------
    // Move (REQ-TRS-DE-004 — reuses PATCH /api/diagrams/layout unchanged)
    // -----------------------------------------------------------------

    /** `MoveAction`'s `toPosition` is the element's new `position`, which in
     * sprotty is always relative to its parent (`MoveMouseListener.
     * createElementMove` adds the drag delta to `element.position`, and
     * `LocationPostprocessor` translates each child by that same local
     * value). So for a nested node the patch is parent-relative — exactly
     * the pin semantics `vis::sprotty` documents, no conversion needed. */
    private handleMoveFinished(moves: ElementMove[]): void {
        const qname = this.currentQname;
        const model = this.activeModel();
        if (!qname || !model || moves.length === 0) {
            return;
        }
        const patch: Record<string, api.LayoutPin> = {};
        for (const move of moves) {
            const node = findShape(model, move.elementId);
            if (!node) {
                // Routing handles and other non-shape moves are not pins.
                continue;
            }
            patch[move.elementId] = { x: Math.round(move.toPosition.x), y: Math.round(move.toPosition.y) };
            node.position = move.toPosition;
        }
        if (Object.keys(patch).length === 0) {
            return;
        }

        api.patchLayout(qname, patch)
            .then(async resp => {
                // `patch_layout` returns a `WriteResponse` (the guarded-write
                // engine), so an always-200 refusal is possible in principle
                // (e.g. a future validation gate on layout): revert on
                // `written:false` too, not just on a network-level throw.
                if (!resp.written) {
                    await this.revertMoves(model, moves);
                    this.toast(`Move failed: ${refusalText(resp)}`);
                } else {
                    for (const id of Object.keys(patch)) {
                        if (!model.pinned.includes(id)) {
                            model.pinned.push(id);
                        }
                    }
                }
            })
            .catch(async err => {
                await this.revertMoves(model, moves);
                this.toast(`Move failed: ${(err as Error).message}`);
            });
    }

    /** Dispatch a compensating move back to each element's prior position
     * (REQ-TRS-DE-005's "disk and diagram must never end up inconsistent"
     * applies to layout too, even though this endpoint's only refusal path
     * today is a network-level failure). */
    private async revertMoves(model: DiagramModelSchema, moves: ElementMove[]): Promise<void> {
        const reverts = moves.filter(m => m.fromPosition).map(m => ({
            elementId: m.elementId,
            toPosition: m.fromPosition!,
            fromPosition: m.toPosition,
        }));
        if (reverts.length === 0) {
            return;
        }
        await this.dispatcher.dispatch(MoveAction.create(reverts, { animate: true, finished: true }));
        for (const r of reverts) {
            const node = findShape(model, r.elementId);
            if (node) {
                node.position = r.toPosition;
            }
        }
    }

    // -----------------------------------------------------------------
    // Pin all / Auto-layout / Save companion SVG (REQ-TRS-VIS-007/011)
    // -----------------------------------------------------------------

    /** Write every placed shape's current position and size as pins in one
     * PATCH. Nothing is optimistic here (the picture does not change), so a
     * refusal only needs reporting; on success the whole diagram counts as
     * pinned, which switches the next layout run to ELK's `fixed` mode. */
    async pinAll(): Promise<void> {
        const qname = this.currentQname;
        const model = this.activeModel();
        if (!qname || !model) {
            return;
        }
        const patch: Record<string, api.LayoutPin> = {};
        for (const shape of allShapes(model)) {
            if (!shape.position) {
                continue;
            }
            const pin: api.LayoutPin = { x: Math.round(shape.position.x), y: Math.round(shape.position.y) };
            if (shape.size && shape.size.width > 0 && shape.size.height > 0) {
                pin.w = Math.round(shape.size.width);
                pin.h = Math.round(shape.size.height);
            }
            patch[shape.id] = pin;
        }
        const count = Object.keys(patch).length;
        if (count === 0) {
            this.toast('Nothing to pin yet — the diagram has not been laid out');
            return;
        }
        try {
            const resp = await api.patchLayout(qname, patch);
            if (!resp.written) {
                this.toast(`Pin all failed: ${refusalText(resp)}`);
                return;
            }
            for (const id of Object.keys(patch)) {
                if (!model.pinned.includes(id)) {
                    model.pinned.push(id);
                }
            }
            // The pinned `w`/`h` is what the server will send back as the
            // shape's size from now on; treat it that way already.
            for (const shape of allShapes(model)) {
                if (shape.size && shape.size.width > 0 && shape.size.height > 0) {
                    shape.serverSize = { ...shape.size };
                    if (shape.type === 'node') {
                        shape.layoutOptions = { ...shape.layoutOptions, resizeContainer: false };
                    }
                }
            }
            this.toast(`Pinned ${count} shapes`, 'info');
        } catch (err) {
            this.toast(`Pin all failed: ${(err as Error).message}`);
        }
    }

    /** Clear every pin on the server, then re-fetch and let ELK lay the
     * diagram out from scratch. The cached model is only dropped once the
     * DELETE was accepted, so a refusal leaves the picture as it was. */
    async autoLayout(): Promise<void> {
        const qname = this.currentQname;
        if (!qname) {
            return;
        }
        try {
            const resp = await api.deleteLayout(qname);
            if (!resp.written) {
                this.toast(`Auto-layout failed: ${refusalText(resp)}`);
                return;
            }
        } catch (err) {
            this.toast(`Auto-layout failed: ${(err as Error).message}`);
            return;
        }
        this.forget(qname);
        await this.activate(qname);
    }

    /** Serialise the live render (`svg-export.ts`) and write it as the
     * diagram's companion SVG through the guarded-write engine. */
    async saveSvg(): Promise<void> {
        const qname = this.currentQname;
        if (!qname) {
            return;
        }
        // sprotty culls shapes outside the canvas at render time, so the DOM
        // only holds what is on screen. Fit the whole diagram into view for
        // one frame, serialise, then put the viewport back where it was.
        const model = this.activeModel();
        const viewport = await this.modelSource.getViewport();
        await this.dispatcher.dispatch(FitToScreenAction.create([], { padding: 0, animate: false }));
        await this.nextFrame();
        const svg = serialiseDiagramSvg(HOST_ID, qname);
        await this.dispatcher.dispatch(
            SetViewportAction.create(model?.id ?? 'sysml-diagram', { scroll: viewport.scroll, zoom: viewport.zoom }, { animate: false }),
        );
        if (!svg) {
            this.toast('No diagram is mounted');
            return;
        }
        try {
            const resp = await api.putSvg(qname, svg);
            if (!resp.written) {
                this.toast(`Save companion SVG failed: ${refusalText(resp)}`);
                return;
            }
            this.toast('Companion SVG saved', 'info');
        } catch (err) {
            this.toast(`Save companion SVG failed: ${(err as Error).message}`);
        }
    }

    // -----------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------

    private handleSelectionChanged(selected: string[], deselected: string[]): void {
        for (const id of selected) {
            this.selectedIds.add(id);
        }
        for (const id of deselected) {
            this.selectedIds.delete(id);
        }
    }

    /** The viewer patches the DOM on the next animation frame after a model
     * or viewport change. */
    private nextFrame(): Promise<void> {
        return new Promise<void>(resolve => window.requestAnimationFrame(() => resolve()));
    }

    private activeModel(): DiagramModelSchema | undefined {
        return this.currentQname ? this.cache.get(this.currentQname) : undefined;
    }

    private nextCascadePosition(model: DiagramModelSchema): { x: number; y: number } {
        const count = allShapes(model).length;
        const step = 24 * (count % 10);
        return { x: 60 + step, y: 60 + step };
    }

    private toast(message: string, level: 'error' | 'info' = 'error'): void {
        const el = document.getElementById('sprotty-toast');
        if (!el) {
            if (level === 'error') {
                console.error('[diagram-editor]', message);
            } else {
                console.info('[diagram-editor]', message);
            }
            return;
        }
        el.textContent = message;
        el.classList.toggle('info', level === 'info');
        el.style.display = 'block';
        window.clearTimeout((el as unknown as { _hideTimer?: number })._hideTimer);
        (el as unknown as { _hideTimer?: number })._hideTimer = window.setTimeout(() => {
            el.style.display = 'none';
        }, level === 'info' ? 3000 : 6000);
    }
}
