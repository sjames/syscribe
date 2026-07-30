// Connect-edge gesture (`REQ-TRS-DE-004`): "drag from one port/shape to
// another" is implemented as click-source-then-click-target rather than a
// literal drag, since sprotty's core (non-GLSP) package ships no built-in
// edge-creation tool — GLSP's `EdgeCreationTool` is exactly the piece
// `ADR-SYS-DE-001` declines to adopt. This is a plain `MouseListener`
// (sprotty's standalone extension point for custom mouse gestures), toggled
// in and out of `MouseTool` by `DiagramEditor.toggleConnectMode` so it
// doesn't fight the default move/select listeners over the same clicks.
import { MouseListener, SModelElementImpl } from 'sprotty';
import { Action, SelectAction } from 'sprotty-protocol';

export class ConnectMouseListener extends MouseListener {
    pendingSourceId: string | null = null;
    onConnected?: (sourceId: string, targetId: string) => void;

    override mouseDown(target: SModelElementImpl, _event: MouseEvent): (Action | Promise<Action>)[] {
        if (target.type !== 'node') {
            return [];
        }
        if (this.pendingSourceId === null) {
            this.pendingSourceId = target.id;
            return [SelectAction.create({ selectedElementsIDs: [target.id] })];
        }
        if (target.id === this.pendingSourceId) {
            return [];
        }
        const sourceId = this.pendingSourceId;
        this.pendingSourceId = null;
        this.onConnected?.(sourceId, target.id);
        return [SelectAction.create({ deselectedElementsIDs: [sourceId] })];
    }

    reset(): void {
        this.pendingSourceId = null;
    }
}
