// Connect-edge gesture (`REQ-TRS-DE-004`): "drag from one port/shape to
// another" is implemented as click-source-then-click-target rather than a
// literal drag, since sprotty's core (non-GLSP) package ships no built-in
// edge-creation tool — GLSP's `EdgeCreationTool` is exactly the piece
// `ADR-SYS-DE-001` declines to adopt. This is a plain `MouseListener`
// (sprotty's standalone extension point for custom mouse gestures), toggled
// in and out of `MouseTool` by `DiagramEditor.toggleConnectMode` so it
// doesn't fight the default move/select listeners over the same clicks.
//
// With the nested graph (`REQ-TRS-VIS-006`) a click usually lands on a
// node's label or compartment child, so the listener walks up to the nearest
// connectable ancestor (`node` or `port`). Port-to-port connects produce an
// edge between the port ids; the port-aware refusal rules of design §6.3 are
// Phase 2.
import { MouseListener, SChildElementImpl, SModelElementImpl } from 'sprotty';
import { Action, SelectAction } from 'sprotty-protocol';

function connectableAncestor(target: SModelElementImpl): SModelElementImpl | undefined {
    let el: SModelElementImpl | undefined = target;
    while (el) {
        if (el.type === 'node' || el.type === 'port') {
            return el;
        }
        el = el instanceof SChildElementImpl ? el.parent : undefined;
    }
    return undefined;
}

export class ConnectMouseListener extends MouseListener {
    pendingSourceId: string | null = null;
    onConnected?: (sourceId: string, targetId: string) => void;

    override mouseDown(target: SModelElementImpl, _event: MouseEvent): (Action | Promise<Action>)[] {
        const shape = connectableAncestor(target);
        if (!shape) {
            return [];
        }
        if (this.pendingSourceId === null) {
            this.pendingSourceId = shape.id;
            return [SelectAction.create({ selectedElementsIDs: [shape.id] })];
        }
        if (shape.id === this.pendingSourceId) {
            return [];
        }
        const sourceId = this.pendingSourceId;
        this.pendingSourceId = null;
        this.onConnected?.(sourceId, shape.id);
        return [SelectAction.create({ deselectedElementsIDs: [sourceId] })];
    }

    reset(): void {
        this.pendingSourceId = null;
    }
}
