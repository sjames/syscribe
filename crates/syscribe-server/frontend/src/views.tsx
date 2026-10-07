/** @jsx svg */
// Views for the nested SysML graph (`ADR-SYS-VIS-001`, `REQ-TRS-VIS-006`):
// one node view that distinguishes containers (boundary) from blocks, a port
// view filled by direction, a label view and a compartment view. Colours are
// chosen by the resolved element type first and the IR role second; Phase 2
// moves this table to `vis::style` in Rust so one visual language is shared
// with the PlantUML/Mermaid/SVG writers.
import { injectable } from 'inversify';
import { VNode } from 'snabbdom';
import {
    IView,
    IViewArgs,
    PolylineEdgeView,
    RenderingContext,
    SCompartmentImpl,
    SEdgeImpl,
    ShapeView,
    SLabelImpl,
    SNodeImpl,
    SPortImpl,
    svg,
} from 'sprotty';
import { Point } from 'sprotty-protocol';
import { isContainerKind, PORT_SIZE } from './layout-shim';

/** Extra fields sprotty's `SModelFactory` copies onto the instance verbatim
 * from the schemas in `types.ts` — not part of the `S*Impl` classes
 * themselves, so views read them through these casts. */
type WithShapeFields = {
    kind: string;
    name: string;
    ref: string;
    resolved?: boolean;
    elementType?: string;
    stereotype?: string;
    isAbstract?: boolean;
    direction?: string;
};
type SysmlNode = SNodeImpl & WithShapeFields;
type SysmlPort = SPortImpl & WithShapeFields;
type SysmlCompartment = SCompartmentImpl & { lines?: string[] };
type WithSysmlEdgeFields = SEdgeImpl & { kind: string };

interface KindStyle {
    fill: string;
    stroke: string;
    headerFill?: string;
}

function nodeStyle(elementType: string | undefined, kind: string): KindStyle {
    switch (elementType) {
        case 'RequirementDef':
        case 'Requirement':
            return { fill: '#f9f7ff', stroke: '#4a0a6e', headerFill: '#4a0a6e' };
        case 'TestCase':
        case 'TestCaseDef':
            return { fill: '#f0fff4', stroke: '#1e6b2e', headerFill: '#1e6b2e' };
        case 'PartDef':
        case 'Part':
            return { fill: '#f5f5fa', stroke: '#3a3a4a' };
    }
    switch (kind) {
        case 'boundary':
        case 'system-boundary':
        case 'swimlane':
        case 'fragment':
            return { fill: '#fafafa', stroke: '#3a3a4a' };
        case 'requirement':
            return { fill: '#f9f7ff', stroke: '#4a0a6e', headerFill: '#4a0a6e' };
        case 'testcase':
            return { fill: '#f0fff4', stroke: '#1e6b2e', headerFill: '#1e6b2e' };
        case 'note':
            return { fill: '#fffbe6', stroke: '#8a7a2a' };
        case 'state':
            return { fill: '#fff7f0', stroke: '#8a4a1e' };
        default:
            return { fill: '#f5f5fa', stroke: '#666' };
    }
}

function edgeStyle(kind: string): { stroke: string; dash?: string } {
    switch (kind) {
        case 'flow':
            return { stroke: '#3a6ea5' };
        case 'binding':
            return { stroke: '#3a6ea5', dash: '5,3' };
        case 'connection':
        case 'succession':
            return { stroke: '#555' };
        case 'inheritance':
        case 'composition':
        case 'aggregation':
        case 'association':
        case 'containment':
            return { stroke: '#333' };
        case 'dependency':
        case 'include':
        case 'extend':
            return { stroke: '#555', dash: '5,3' };
        case 'derive':
        case 'refine':
        case 'trace':
        case 'copy':
            return { stroke: '#555', dash: '5,3' };
        case 'satisfy':
        case 'verify':
            return { stroke: '#3a6ea5', dash: '5,3' };
        case 'allocation':
            return { stroke: '#7a3ea5', dash: '3,3' };
        default:
            return { stroke: '#888' };
    }
}

@injectable()
export class SysmlNodeView extends ShapeView implements IView {
    render(node: Readonly<SNodeImpl>, context: RenderingContext, _args?: IViewArgs): VNode | undefined {
        if (!this.isVisible(node, context)) {
            return undefined;
        }
        const n = node as Readonly<SysmlNode>;
        const width = Math.max(n.size?.width ?? 0, 0) || 160;
        const height = Math.max(n.size?.height ?? 0, 0) || 50;
        const container = isContainerKind(n.kind);
        const style = nodeStyle(n.elementType, n.kind);
        // `selected` also doubles as the connect-mode "pending source" highlight
        // (`ConnectMouseListener` dispatches a plain `SelectAction`) — one less
        // bespoke visual state to wire up for a gesture that's inherently
        // transient (cleared as soon as the second node is clicked).
        const selected = !!n.selected;
        const unresolved = n.resolved === false;
        const outlineWidth = selected ? 2.5 : container ? 1.2 : 1.5;
        const outlineColor = selected ? '#1d4ed8' : style.stroke;
        const stereotype = n.stereotype ? `«${n.stereotype}»` : undefined;

        return (
            <g class-sysml-node={true} class-selected={selected} class-unresolved={unresolved}>
                <rect
                    x={0}
                    y={0}
                    width={width}
                    height={height}
                    rx={container ? 8 : 4}
                    fill={style.fill}
                    stroke={outlineColor}
                    stroke-width={outlineWidth}
                    stroke-dasharray={unresolved ? '6,3' : undefined}
                />
                {style.headerFill && !container && (
                    <rect x={0} y={0} width={width} height={18} rx={4} fill={style.headerFill} opacity={0.12} />
                )}
                {stereotype && !container && (
                    <text x={width / 2} y={13} text-anchor="middle" font-size={9} fill={style.stroke} font-style="italic">
                        {stereotype}
                    </text>
                )}
                {stereotype && container && (
                    <text x={12} y={30} font-size={9} fill={style.stroke} font-style="italic">
                        {stereotype}
                    </text>
                )}
                {n.isAbstract && (
                    <text
                        x={container ? 12 : width / 2}
                        y={container ? 42 : height - 4}
                        text-anchor={container ? 'start' : 'middle'}
                        font-size={9}
                        fill="#666"
                        font-style="italic"
                    >
                        isAbstract
                    </text>
                )}
                {context.renderChildren(node)}
            </g>
        );
    }
}

/** A 12×12 square on its parent's border, filled by direction:
 * `in` white, `out` dark, `inout` half and half. */
@injectable()
export class SysmlPortView extends ShapeView implements IView {
    render(port: Readonly<SPortImpl>, context: RenderingContext, _args?: IViewArgs): VNode | undefined {
        if (!this.isVisible(port, context)) {
            return undefined;
        }
        const p = port as Readonly<SysmlPort>;
        const size = p.size?.width > 0 ? p.size.width : PORT_SIZE;
        const selected = !!p.selected;
        const unresolved = p.resolved === false;
        const dark = '#333';
        const fill = p.direction === 'out' ? dark : '#fff';
        return (
            <g class-sysml-port={true} class-selected={selected}>
                <rect
                    x={0}
                    y={0}
                    width={size}
                    height={size}
                    fill={fill}
                    stroke={selected ? '#1d4ed8' : dark}
                    stroke-width={selected ? 2 : 1.2}
                    stroke-dasharray={unresolved ? '2,2' : undefined}
                />
                {p.direction === 'inout' && <polygon points={`0,0 ${size},0 0,${size}`} fill={dark} />}
                {context.renderChildren(port)}
            </g>
        );
    }
}

/** The name label of a node/port (its `<id>-label` child) or a free label
 * shape. Anchored by what it belongs to: left in a container, centred on a
 * block, centred above a port. */
@injectable()
export class SysmlLabelView extends ShapeView implements IView {
    render(label: Readonly<SLabelImpl>, _context: RenderingContext, _args?: IViewArgs): VNode | undefined {
        const parent = label.parent as unknown as Partial<WithShapeFields> & { type?: string };
        const onPort = parent.type === 'port';
        const inContainer = parent.kind !== undefined && isContainerKind(parent.kind);
        const fontSize = onPort ? 8 : 12;
        return (
            <text
                class-sysml-label={true}
                text-anchor={inContainer ? 'start' : 'middle'}
                font-size={fontSize}
                font-weight={onPort ? 'normal' : 'bold'}
                fill="#222"
            >
                {label.text}
            </text>
        );
    }
}

/** A compartment: a separator line and one text row per `lines` entry. */
@injectable()
export class SysmlCompartmentView implements IView {
    render(compartment: Readonly<SCompartmentImpl>, context: RenderingContext, _args?: IViewArgs): VNode | undefined {
        const c = compartment as Readonly<SysmlCompartment>;
        const width = c.size?.width > 0 ? c.size.width : 160;
        const lines = c.lines ?? [];
        return (
            <g class-sysml-compartment={true}>
                <line x1={0} y1={0} x2={width} y2={0} stroke="#888" stroke-width={1} />
                {lines.map((line, i) => (
                    <text x={6} y={12 + i * 14} font-size={10} fill="#333">
                        {line}
                    </text>
                ))}
                {context.renderChildren(compartment)}
            </g>
        );
    }
}

@injectable()
export class SysmlEdgeView extends PolylineEdgeView {
    protected override renderLine(
        edge: Readonly<SEdgeImpl>,
        segments: Point[],
        context: RenderingContext,
        args?: IViewArgs,
    ): VNode {
        const vnode = super.renderLine(edge, segments, context, args);
        const kind = (edge as Readonly<WithSysmlEdgeFields>).kind ?? '';
        const style = edgeStyle(kind);
        vnode.data = vnode.data ?? {};
        vnode.data.attrs = {
            ...(vnode.data.attrs ?? {}),
            fill: 'none',
            stroke: style.stroke,
            'stroke-width': 1.4,
            ...(style.dash ? { 'stroke-dasharray': style.dash } : {}),
        };
        return vnode;
    }
}
