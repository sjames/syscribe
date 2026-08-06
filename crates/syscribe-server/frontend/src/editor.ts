// Orchestrates the sprotty editor for one persistent host div, across
// however many diagram tabs `base.html`'s tab bar opens (`ADR-SYS-DE-001`,
// `REQ-TRS-DE-004`). One `DiagramEditor` instance, one DI container, one
// `LocalModelSource` — switching tabs calls `setModel(...)` with a different
// cached schema rather than tearing anything down, since sprotty's
// `ModelViewer` patches a *specific* DOM node by id (`viewerOptions.baseDiv`)
// and losing that node (e.g. via an `innerHTML` reset elsewhere) would break
// future patches — see the code comment on `HOST_ID` below.
import 'reflect-metadata';
import { Container } from 'inversify';
import { IActionDispatcher, LocalModelSource, MouseTool, MoveMouseListener, SelectMouseListener, TYPES } from 'sprotty';
import { CreateElementAction, DeleteElementAction, ElementMove, MoveAction } from 'sprotty-protocol';
import * as api from './api';
import { createDiagramContainer } from './container';
import { ConnectMouseListener } from './connect-listener';
import { DiagramModelSchema, Finding, isEdgeSchema, isNodeSchema, SysmlChildSchema, SysmlNodeSchema } from './types';

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
        const schema: SysmlNodeSchema = {
            id: shapeId,
            type: 'node',
            ref,
            kind,
            name: nodeName(ref),
            position,
            size: { width: 200, height: 50 },
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
            this.removeLocal(model, [shapeId]);
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
        const node = model.children.find(c => c.id === shapeId && isNodeSchema(c)) as SysmlNodeSchema | undefined;
        if (!node) {
            return;
        }
        const connectedEdgeIds = model.children
            .filter(isEdgeSchema)
            .filter(e => e.sourceId === shapeId || e.targetId === shapeId)
            .map(e => e.id);
        const removedIds = [shapeId, ...connectedEdgeIds];
        const removedSchemas = model.children.filter(c => removedIds.includes(c.id));

        // Optimistic apply.
        this.removeLocal(model, removedIds);
        await this.dispatcher.dispatch(DeleteElementAction.create(removedIds));

        const resp = await api.deleteElement(node.ref);
        if (resp.written) {
            return;
        }

        // Revert: disk is unchanged (REQ-TRS-DE-005), so put every removed
        // schema (the node and its dangling edges) back exactly as it was.
        model.children.push(...removedSchemas);
        for (const schema of removedSchemas) {
            await this.dispatcher.dispatch(CreateElementAction.create(schema, { containerId: model.id }));
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

    private async handleConnect(sourceShapeId: string, targetShapeId: string): Promise<void> {
        const qname = this.currentQname;
        const model = this.activeModel();
        if (!qname || !model) {
            return;
        }
        const source = model.children.find(c => c.id === sourceShapeId && isNodeSchema(c)) as
            | SysmlNodeSchema
            | undefined;
        const target = model.children.find(c => c.id === targetShapeId && isNodeSchema(c)) as
            | SysmlNodeSchema
            | undefined;
        if (!source || !target) {
            return;
        }

        const edgeId = `e-${sourceShapeId}-${targetShapeId}-${Date.now().toString(36)}`;
        const schema: SysmlChildSchema = {
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
            this.removeLocal(model, [edgeId]);
            await this.dispatcher.dispatch(DeleteElementAction.create([edgeId]));
            this.toast(`Connect failed: ${resp.reason ?? summarizeFindings(resp.newErrors)}`);
        }
    }

    // -----------------------------------------------------------------
    // Move (REQ-TRS-DE-004 — reuses PATCH /api/diagrams/layout unchanged)
    // -----------------------------------------------------------------

    private handleMoveFinished(moves: ElementMove[]): void {
        const qname = this.currentQname;
        const model = this.activeModel();
        if (!qname || !model || moves.length === 0) {
            return;
        }
        const patch: Record<string, { x: number; y: number }> = {};
        for (const move of moves) {
            patch[move.elementId] = { x: Math.round(move.toPosition.x), y: Math.round(move.toPosition.y) };
            const node = model.children.find(c => c.id === move.elementId && isNodeSchema(c)) as
                | SysmlNodeSchema
                | undefined;
            if (node) {
                node.position = move.toPosition;
            }
        }

        api.patchLayout(qname, patch)
            .then(async resp => {
                // `patch_layout` now returns a `WriteResponse` (it moved onto
                // the guarded-write engine) instead of the always-`{ok:true}`
                // body the retired `routes::write::patch_layout` returned —
                // an always-200 refusal is now possible in principle (e.g. a
                // future validation gate on layout), so revert on
                // `written:false` too, not just on a network-level throw.
                if (!resp.written) {
                    await this.revertMoves(model, moves);
                    this.toast(`Move failed: ${resp.reason ?? summarizeFindings(resp.newErrors)}`);
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
            const node = model.children.find(c => c.id === r.elementId && isNodeSchema(c)) as
                | SysmlNodeSchema
                | undefined;
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

    private removeLocal(model: DiagramModelSchema, ids: string[]): void {
        model.children = model.children.filter(c => !ids.includes(c.id));
    }

    private nextCascadePosition(model: DiagramModelSchema): { x: number; y: number } {
        const count = model.children.filter(isNodeSchema).length;
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
