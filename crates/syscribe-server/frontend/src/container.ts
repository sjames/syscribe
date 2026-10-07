// Sprotty DI container wiring (`ADR-SYS-DE-001` point 4 — sprotty standalone,
// not GLSP). One container/`LocalModelSource` is created once and reused for
// every diagram tab (`editor.ts` calls `LocalModelSource.setModel(...)` when
// the active diagram changes) rather than rebuilding the whole DI graph per
// diagram open.
import { Container } from 'inversify';
import {
    boundsModule,
    configureCommand,
    configureModelElement,
    configureViewerOptions,
    CreateElementCommand,
    defaultModule,
    DeleteElementCommand,
    fadeModule,
    hoverModule,
    LocalModelSource,
    modelSourceModule,
    moveModule,
    onAction,
    routingModule,
    SCompartmentImpl,
    SEdgeImpl,
    selectModule,
    SGraphImpl,
    SGraphView,
    SLabelImpl,
    SNodeImpl,
    SPortImpl,
    TYPES,
    undoRedoModule,
    updateModule,
    viewportModule,
    zorderModule,
} from 'sprotty';
import { Action, ElementMove, MoveAction, SelectAction } from 'sprotty-protocol';
import { SysmlCompartmentView, SysmlEdgeView, SysmlLabelView, SysmlNodeView, SysmlPortView } from './views';

export interface DiagramCallbacks {
    /** Fired once per completed drag (`MoveAction.finished`), REQ-TRS-DE-004's move gesture. */
    onMoveFinished(moves: ElementMove[]): void;
    /** Fired on every selection change — `DiagramEditor` uses this to track what `deleteSelected()` should remove. */
    onSelectionChanged(selectedIds: string[], deselectedIds: string[]): void;
}

export function createDiagramContainer(hostDivId: string, callbacks: DiagramCallbacks): Container {
    const container = new Container();
    container.load(
        defaultModule,
        boundsModule,
        moveModule,
        selectModule,
        viewportModule,
        updateModule,
        undoRedoModule,
        zorderModule,
        hoverModule,
        fadeModule,
        routingModule,
        modelSourceModule,
    );

    container.bind(LocalModelSource).toSelf().inSingletonScope();
    container.bind(TYPES.ModelSource).toService(LocalModelSource);

    // No hidden-render measuring pass in Phase 0: the endpoint supplies
    // `position`/`size` for pinned nodes only (`REQ-TRS-VIS-006`) and
    // `layout-shim.ts` fills in the rest before `setModel`, so
    // `LocalModelSource.submitModel` can skip the `RequestBoundsAction` round
    // trip. Phase 2 (`REQ-TRS-VIS-007`) turns `needsClientLayout` on so label
    // bounds are measured before ELK runs.
    configureViewerOptions(container, {
        baseDiv: hostDivId,
        hiddenDiv: hostDivId + '-hidden',
        needsClientLayout: false,
        needsServerLayout: false,
    });

    // The nested schema (`vis::sprotty`): nodes contain ports, a name label,
    // compartments and nested nodes; edges stay at the root.
    configureModelElement(container, 'graph', SGraphImpl, SGraphView);
    configureModelElement(container, 'node', SNodeImpl, SysmlNodeView);
    configureModelElement(container, 'port', SPortImpl, SysmlPortView);
    configureModelElement(container, 'label', SLabelImpl, SysmlLabelView);
    configureModelElement(container, 'compartment', SCompartmentImpl, SysmlCompartmentView);
    configureModelElement(container, 'edge', SEdgeImpl, SysmlEdgeView);

    // Registered so `DiagramEditor` can dispatch `CreateElementAction`/
    // `DeleteElementAction` for the optimistic local apply of create-node,
    // delete-node, and connect-edge (an edge is just another schema element
    // added to the same container) — see that module's module doc comment.
    configureCommand(container, CreateElementCommand);
    configureCommand(container, DeleteElementCommand);

    // Move and selection are both gestures sprotty's own `moveModule`/
    // `selectModule` already provide (drag-to-move, click-to-select); these
    // two `onAction` registrations are the "custom action handler intercepts
    // it" side-channel `ADR-SYS-DE-001` describes — they run *alongside* the
    // built-in command handlers (the action-handler registry is
    // multi-bound), observing the same actions to drive the REST calls
    // without altering how the gesture itself renders.
    onAction(container, MoveAction.KIND, (action: Action) => {
        const move = action as MoveAction;
        if (move.finished) {
            callbacks.onMoveFinished(move.moves);
        }
    });
    onAction(container, SelectAction.KIND, (action: Action) => {
        const select = action as SelectAction;
        callbacks.onSelectionChanged(select.selectedElementsIDs, select.deselectedElementsIDs);
    });

    return container;
}
