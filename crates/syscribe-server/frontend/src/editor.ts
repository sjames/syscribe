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
import 'reflect-metadata';
import { Container } from 'inversify';
import { IActionDispatcher, LocalModelSource, MouseTool, MoveMouseListener, SelectMouseListener, TYPES } from 'sprotty';
import { CreateElementAction, DeleteElementAction, ElementMove, MoveAction } from 'sprotty-protocol';
import * as api from './api';
import { createDiagramContainer } from './container';
import { ConnectMouseListener } from './connect-listener';
import { applyPhase0Layout, defaultSize } from './layout-shim';
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
} from './types';

/** Id of the persistent DOM div sprotty renders into — see `index.html`. It
 * must never be recreated (no `innerHTML = ...` on it or an ancestor) for as
 * long as this editor instance lives, or sprotty's snabbdom patcher loses its
 * reference to the live DOM node and stops updating anything. */
const HOST_ID = 'sprotty-host';

function summarizeFindings(findings: Finding[]): string {
    return findings.map(f => `${f.code}: ${f.message}`).join('; ');
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
            // Phase 0: place everything the server left unpinned (ELK in Phase 2).
            applyPhase0Layout(model);
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
        // child mirrors what `vis::sprotty` would emit on the next fetch.
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
            children: [{ id: `${shapeId}-label`, type: 'label', text: name, position: { x: 80, y: 32 } }],
        };

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
            this.toast(`Create failed: ${resp.reason ?? summarizeFindings(resp.newErrors)}`);
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
            this.toast(`Delete failed: ${resp.reason ?? summarizeFindings(resp.newErrors)}`);
        }
    }

    // -----------------------------------------------------------------
    // Connect edge (REQ-TRS-DE-004)
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

    /** Both ends may be nodes or ports anywhere in the tree; the edge joins
     * their ids and sits at the root, where sprotty resolves them. */
    private async handleConnect(sourceShapeId: string, targetShapeId: string): Promise<void> {
        const qname = this.currentQname;
        const model = this.activeModel();
        if (!qname || !model) {
            return;
        }
        const source = findShape(model, sourceShapeId);
        const target = findShape(model, targetShapeId);
        if (!source || !target) {
            return;
        }

        const edgeId = `e-${sourceShapeId}-${targetShapeId}-${Date.now().toString(36)}`;
        const schema: SysmlEdgeSchema = {
            id: edgeId,
            type: 'edge',
            sourceId: sourceShapeId,
            targetId: targetShapeId,
            kind: 'connection',
        };

        // Optimistic apply.
        model.children.push(schema);
        await this.dispatcher.dispatch(CreateElementAction.create(schema, { containerId: model.id }));

        // The diagram's `subject:` is the natural "owning element" for a
        // connect gesture's `connections:` mutation (see
        // `routes::diagram_model`'s doc comment) — the diagram itself has no
        // `connections:` list of its own. Falls back to the diagram's own
        // qname if `subject` is unset, matching `AddConnectionRequest.qname`
        // resolving through the same `Resolver` either way.
        const ownerQname = model.subject ?? qname;
        const resp = await api.addConnection({
            qname: ownerQname,
            from: source.ref,
            to: target.ref,
            diagram: { qname, edgeId, sourceShapeId, targetShapeId },
        });

        if (!resp.written) {
            removeFromTree(model, [edgeId]);
            await this.dispatcher.dispatch(DeleteElementAction.create([edgeId]));
            this.toast(`Connect failed: ${resp.reason ?? summarizeFindings(resp.newErrors)}`);
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
        const patch: Record<string, { x: number; y: number }> = {};
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
                    this.toast(`Move failed: ${resp.reason ?? summarizeFindings(resp.newErrors)}`);
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

    private activeModel(): DiagramModelSchema | undefined {
        return this.currentQname ? this.cache.get(this.currentQname) : undefined;
    }

    private nextCascadePosition(model: DiagramModelSchema): { x: number; y: number } {
        const count = allShapes(model).length;
        const step = 24 * (count % 10);
        return { x: 60 + step, y: 60 + step };
    }

    private toast(message: string): void {
        const el = document.getElementById('sprotty-toast');
        if (!el) {
            console.error('[diagram-editor]', message);
            return;
        }
        el.textContent = message;
        el.style.display = 'block';
        window.clearTimeout((el as unknown as { _hideTimer?: number })._hideTimer);
        (el as unknown as { _hideTimer?: number })._hideTimer = window.setTimeout(() => {
            el.style.display = 'none';
        }, 6000);
    }
}
