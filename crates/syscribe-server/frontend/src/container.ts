// Sprotty DI container wiring (`ADR-SYS-DE-001` point 4 — sprotty standalone,
// not GLSP). One container/`LocalModelSource` is created once and reused for
// every diagram tab (`editor.ts` calls `LocalModelSource.setModel(...)` when
// the active diagram changes) rather than rebuilding the whole DI graph per
// diagram open.
import { Container, injectable } from 'inversify';
import ElkConstructor from 'elkjs/lib/elk.bundled';
import {
    boundsModule,
    configureCommand,
    configureModelElement,
    configureViewerOptions,
    ConsoleLogger,
    CreateElementCommand,
    defaultModule,
    DeleteElementCommand,
    fadeModule,
    HiddenBoundsUpdater,
    hoverModule,
    InternalBoundsAware,
    LocalModelSource,
    LogLevel,
    modelSourceModule,
    moveModule,
    onAction,
    routingModule,
    SCompartmentImpl,
    SEdgeImpl,
    selectModule,
    SGraphImpl,
    SLabelImpl,
    SModelElementImpl,
    SNodeImpl,
    SPortImpl,
    TYPES,
    undoRedoModule,
    updateModule,
    viewportModule,
    zorderModule,
} from 'sprotty';
import { Bounds } from 'sprotty-protocol';
import {
    ElkFactory,
    ElkLayoutEngine,
    elkLayoutModule,
    ILayoutConfigurator,
    ILayoutPostprocessor,
    ILayoutPreprocessor,
} from 'sprotty-elk/lib/inversify';
import { Action, ElementMove, MoveAction, SelectAction } from 'sprotty-protocol';
import { LayoutState, serverSizeOf, SizedSchema, SyscribeLayoutConfigurator, SyscribeLayoutProcessor } from './layout';
import {
    SysmlCompartmentView,
    SysmlEdgeLabelImpl,
    SysmlEdgeView,
    SysmlGraphView,
    SysmlLabelView,
    SysmlNodeView,
    SysmlPortView,
} from './views';

/** The hidden measuring pass, with server sizes left alone (`layout.ts`'s
 * module doc, `REQ-TRS-VIS-017`): an element that carries `serverSize`
 * reports that width/height instead of its `getBBox()` extent. The measured
 * `x`/`y` are kept — for a `<text>` they are the glyph box's offset from the
 * anchor, which sprotty turns into the label's alignment so its top-left
 * corner lands on `position`. With the reported size equal to the element's
 * bounds, `HiddenBoundsUpdater` records no change and the size survives
 * the pass; the `vbox` layouter, run next, is told not to resize such
 * containers (`resizeContainer: false`), so it only positions their
 * children inside the carried size. */
@injectable()
export class SyscribeHiddenBoundsUpdater extends HiddenBoundsUpdater {
    protected override getBounds(elm: Node, element: SModelElementImpl & InternalBoundsAware): Bounds {
        const measured = super.getBounds(elm, element);
        const server = serverSizeOf(element as unknown as SizedSchema);
        if (!server) {
            return measured;
        }
        return { x: measured.x, y: measured.y, width: server.width, height: server.height };
    }
}

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
        elkLayoutModule,
    );

    container.bind(LocalModelSource).toSelf().inSingletonScope();
    container.bind(TYPES.ModelSource).toService(LocalModelSource);

    // sprotty's default is a `NullLogger`, which swallows a failed layout
    // run (`LocalModelSource.doSubmitModel` catches and logs) and leaves a
    // blank canvas with no trace; surface errors and warnings on the console.
    container.rebind(TYPES.ILogger).to(ConsoleLogger).inSingletonScope();
    container.rebind(TYPES.LogLevel).toConstantValue(LogLevel.warn);

    // ELK in the browser (`REQ-TRS-VIS-007`, design §6.2): the bundled elkjs
    // build runs on the main thread; `sprotty-elk` transforms the sprotty
    // graph, our configurator/processor pair (`layout.ts`) maps the server's
    // `layoutOptions`, pins and reversed edge kinds onto it.
    const layoutState = new LayoutState();
    container.bind(ElkFactory).toConstantValue(() => new ElkConstructor());
    container.rebind(ILayoutConfigurator).toConstantValue(new SyscribeLayoutConfigurator(layoutState));
    const processor = new SyscribeLayoutProcessor(layoutState);
    container.bind(ILayoutPreprocessor).toConstantValue(processor);
    container.bind(ILayoutPostprocessor).toConstantValue(processor);
    container.bind(TYPES.IModelLayoutEngine).toService(ElkLayoutEngine);

    // `needsClientLayout: true` makes `LocalModelSource.submitModel` run the
    // hidden measuring pass (`RequestBoundsAction` → `ComputedBoundsAction`)
    // first, so every label has real bounds and every `vbox` node a measured
    // size before the layout engine sees the graph — except what the server
    // already sized (`SyscribeHiddenBoundsUpdater`; `boundsModule` binds the
    // base class to itself and aliases `TYPES.HiddenVNodePostprocessor` to
    // it, so rebinding the class is enough).
    container.rebind(HiddenBoundsUpdater).to(SyscribeHiddenBoundsUpdater).inSingletonScope();
    configureViewerOptions(container, {
        baseDiv: hostDivId,
        hiddenDiv: hostDivId + '-hidden',
        needsClientLayout: true,
        needsServerLayout: false,
    });

    // The nested schema (`vis::sprotty`): nodes contain ports, a name label,
    // compartments and nested nodes; edges stay at the root.
    configureModelElement(container, 'graph', SGraphImpl, SysmlGraphView);
    configureModelElement(container, 'node', SNodeImpl, SysmlNodeView);
    configureModelElement(container, 'port', SPortImpl, SysmlPortView);
    configureModelElement(container, 'label', SLabelImpl, SysmlLabelView);
    // An edge's keyword/label children are placed by ELK (absolute, root
    // coordinates). Plain `SLabelImpl` carries `edgeLayoutFeature`, which
    // makes `LocationPostprocessor` skip them in favour of the (unloaded)
    // edge-layout module, so they get a class without that feature.
    configureModelElement(container, 'label:edge', SysmlEdgeLabelImpl, SysmlLabelView);
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
