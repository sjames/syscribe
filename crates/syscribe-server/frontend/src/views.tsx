/** @jsx svg */
// Views for the nested SysML graph (`ADR-SYS-VIS-001`, `REQ-TRS-VIS-006`).
// The visual language is owned by `vis::style` in Rust (`REQ-TRS-VIS-012`):
// every node, port and edge arrives with a resolved `style`, and the views
// read it. The small tables at the top are only a fallback for an element
// without one (an older server, or a shape created locally by `editor.ts`
// before the next fetch), so the picture never degrades to "no colours".
//
// Each node/port `<g>` carries `data-sysml-ref` and each edge `<g>`
// `data-sysml-ref`/`data-sysml-source`/`data-sysml-target`; `svg-export.ts`
// turns those into the `sysml:*` attributes of spec §8.16.5 when the user
// presses *Save companion SVG*.
import { injectable } from 'inversify';
import { VNode } from 'snabbdom';
import {
    edgeLayoutFeature,
    IView,
    IViewArgs,
    PolylineEdgeView,
    RenderingContext,
    SCompartmentImpl,
    SEdgeImpl,
    SGraphImpl,
    SGraphView,
    ShapeView,
    SLabelImpl,
    SNodeImpl,
    SPortImpl,
    svg,
} from 'sprotty';

import { Point } from 'sprotty-protocol';
import { isContainerKind, PORT_SIZE } from './layout';
import { ArrowHead, EdgeStyle, LabelRole, NodeStyle, PortStyle } from './types';

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
type SysmlNode = SNodeImpl & WithShapeFields & { style?: NodeStyle; banners?: string[] };
type SysmlPort = SPortImpl & WithShapeFields & { style?: PortStyle };
type SysmlLabel = SLabelImpl & { role?: LabelRole };
type SysmlEdge = SEdgeImpl & { kind: string; style?: EdgeStyle; ref?: string; label?: string };

// ---------------------------------------------------------------------------
// Fallback tables (used only when the server sent no `style`)
// ---------------------------------------------------------------------------

function fallbackNodeStyle(elementType: string | undefined, kind: string): NodeStyle {
    switch (elementType) {
        case 'RequirementDef':
        case 'Requirement':
            return { fill: '#f9f7ff', stroke: '#4a0a6e', headerFill: '#4a0a6e', text: '#222' };
        case 'TestCase':
        case 'TestCaseDef':
            return { fill: '#f0fff4', stroke: '#1e6b2e', headerFill: '#1e6b2e', text: '#222' };
        case 'PartDef':
        case 'Part':
            return { fill: '#f5f5fa', stroke: '#3a3a4a', text: '#222' };
    }
    switch (kind) {
        case 'boundary':
        case 'system-boundary':
        case 'swimlane':
        case 'fragment':
            return { fill: '#fafafa', stroke: '#3a3a4a', text: '#222' };
        case 'requirement':
            return { fill: '#f9f7ff', stroke: '#4a0a6e', headerFill: '#4a0a6e', text: '#222' };
        case 'testcase':
            return { fill: '#f0fff4', stroke: '#1e6b2e', headerFill: '#1e6b2e', text: '#222' };
        case 'note':
            return { fill: '#fffbe6', stroke: '#8a7a2a', text: '#222' };
        case 'state':
            return { fill: '#fff7f0', stroke: '#8a4a1e', text: '#222' };
        default:
            return { fill: '#f5f5fa', stroke: '#666', text: '#222' };
    }
}

function fallbackPortStyle(direction: string | undefined): PortStyle {
    switch (direction) {
        case 'out':
            return { fill: '#333', stroke: '#333', glyph: 'out' };
        case 'in':
            return { fill: '#fff', stroke: '#333', glyph: 'in' };
        case 'inout':
            return { fill: '#fff', stroke: '#333', glyph: 'inout' };
        default:
            return { fill: '#ddd', stroke: '#333', glyph: 'none' };
    }
}

function fallbackEdgeStyle(kind: string): EdgeStyle {
    switch (kind) {
        case 'flow':
            return { stroke: '#3a6ea5', arrowTarget: 'filled' };
        case 'binding':
            return { stroke: '#3a6ea5', dash: '5,3', keyword: '=' };
        case 'inheritance':
            return { stroke: '#333', arrowTarget: 'hollowTriangle' };
        case 'composition':
            return { stroke: '#333', arrowSource: 'filledDiamond' };
        case 'aggregation':
            return { stroke: '#333', arrowSource: 'hollowDiamond' };
        case 'dependency':
        case 'include':
        case 'extend':
        case 'derive':
        case 'refine':
        case 'trace':
        case 'copy':
        case 'satisfy':
        case 'verify':
            return { stroke: '#555', dash: '5,3', arrowTarget: 'open', keyword: kind };
        case 'allocation':
            return { stroke: '#7a3ea5', dash: '3,3', arrowTarget: 'open', keyword: 'allocate' };
        default:
            return { stroke: '#555' };
    }
}

// ---------------------------------------------------------------------------
// Arrowhead markers
// ---------------------------------------------------------------------------

/** One `<marker>` per (arrowhead, colour) pair actually used by the graph's
 * edges — markers cannot inherit the referencing path's stroke, so the
 * colour is baked into the id. */
export function markerId(arrow: ArrowHead, stroke: string): string {
    return `sysml-arrow-${arrow}-${stroke.replace(/[^A-Za-z0-9]/g, '')}`;
}

function markerDef(arrow: ArrowHead, stroke: string): VNode | undefined {
    const id = markerId(arrow, stroke);
    const common = { id, orient: 'auto-start-reverse', markerUnits: 'userSpaceOnUse' };
    switch (arrow) {
        case 'filled':
            return (
                <marker {...common} markerWidth={10} markerHeight={8} refX={10} refY={4}>
                    <path d="M0,0 L10,4 L0,8 z" fill={stroke} stroke="none" />
                </marker>
            );
        case 'open':
            return (
                <marker {...common} markerWidth={11} markerHeight={10} refX={10} refY={5}>
                    <path d="M0,0 L10,5 L0,10" fill="none" stroke={stroke} stroke-width={1.4} />
                </marker>
            );
        case 'hollowTriangle':
            return (
                <marker {...common} markerWidth={14} markerHeight={12} refX={13} refY={6}>
                    <path d="M0.7,0.7 L13,6 L0.7,11.3 z" fill="#fff" stroke={stroke} stroke-width={1.4} />
                </marker>
            );
        case 'filledDiamond':
            return (
                <marker {...common} markerWidth={16} markerHeight={10} refX={15} refY={5}>
                    <path d="M1,5 L8,1 L15,5 L8,9 z" fill={stroke} stroke={stroke} stroke-width={1} />
                </marker>
            );
        case 'hollowDiamond':
            return (
                <marker {...common} markerWidth={16} markerHeight={10} refX={15} refY={5}>
                    <path d="M1,5 L8,1 L15,5 L8,9 z" fill="#fff" stroke={stroke} stroke-width={1.2} />
                </marker>
            );
        case 'filledCircle':
            return (
                <marker {...common} markerWidth={10} markerHeight={10} refX={9} refY={5}>
                    <circle cx={5} cy={5} r={4} fill={stroke} stroke="none" />
                </marker>
            );
        default:
            return undefined;
    }
}

function edgeStyleOf(edge: Readonly<SysmlEdge>): EdgeStyle {
    return edge.style ?? fallbackEdgeStyle(edge.kind ?? '');
}

/** The root view: sprotty's graph plus a `<defs>` holding every marker the
 * edges reference. Skipped in the hidden measuring pass — a marker defined
 * inside the hidden SVG would shadow the visible one by id. */
@injectable()
export class SysmlGraphView extends SGraphView {
    override render(model: Readonly<SGraphImpl>, context: RenderingContext): VNode {
        const edgeRouting = this.edgeRouterRegistry.routeAllChildren(model);
        const transform = `scale(${model.zoom}) translate(${-model.scroll.x},${-model.scroll.y})`;
        const defs = context.targetKind === 'hidden' ? undefined : this.renderDefs(model);
        return (
            <svg class-sprotty-graph={true}>
                {defs}
                <g transform={transform}>{context.renderChildren(model, { edgeRouting })}</g>
            </svg>
        );
    }

    protected renderDefs(model: Readonly<SGraphImpl>): VNode {
        const seen = new Map<string, VNode>();
        for (const child of model.children) {
            if (!(child instanceof SEdgeImpl)) {
                continue;
            }
            const style = edgeStyleOf(child as SysmlEdge);
            for (const arrow of [style.arrowTarget, style.arrowSource]) {
                if (!arrow || arrow === 'none') {
                    continue;
                }
                const id = markerId(arrow, style.stroke);
                if (!seen.has(id)) {
                    const def = markerDef(arrow, style.stroke);
                    if (def) {
                        seen.set(id, def);
                    }
                }
            }
        }
        return <defs class-sysml-markers={true}>{[...seen.values()]}</defs>;
    }
}

// ---------------------------------------------------------------------------
// Nodes, ports, labels, compartments
// ---------------------------------------------------------------------------

function addClasses(vnode: VNode, names: string[]): VNode {
    vnode.data = vnode.data ?? {};
    const cls = (vnode.data.class = vnode.data.class ?? {});
    for (const n of names) {
        if (n) {
            cls[n] = true;
        }
    }
    return vnode;
}

function labelStackHeight(node: Readonly<SNodeImpl>): number {
    let bottom = 0;
    for (const c of node.children) {
        if (c instanceof SLabelImpl) {
            bottom = Math.max(bottom, c.bounds.y + c.bounds.height);
        }
    }
    return bottom;
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
        const style = n.style ?? fallbackNodeStyle(n.elementType, n.kind);
        // `selected` also doubles as the connect-mode "pending source" highlight
        // (`ConnectMouseListener` dispatches a plain `SelectAction`).
        const selected = !!n.selected;
        const unresolved = n.resolved === false;
        const dashed = unresolved || !!style.dashed;
        const outlineWidth = selected ? 2.5 : container ? 1.2 : 1.5;
        const outlineColor = selected ? '#1d4ed8' : style.stroke;
        const header = style.headerFill && !container ? labelStackHeight(n) + 4 : 0;

        const vnode = (
            <g
                class-sysml-node={true}
                class-selected={selected}
                class-unresolved={unresolved}
                class-abstract={!!n.isAbstract}
                data-sysml-ref={n.ref}
            >
                <rect
                    x={0}
                    y={0}
                    width={width}
                    height={height}
                    rx={container ? 8 : 4}
                    fill={style.fill}
                    stroke={outlineColor}
                    stroke-width={outlineWidth}
                    stroke-dasharray={dashed ? '6,3' : undefined}
                />
                {header > 0 && (
                    <rect x={0} y={0} width={width} height={header} rx={4} fill={style.headerFill as string} opacity={0.14} />
                )}
                {context.renderChildren(node)}
            </g>
        );
        return addClasses(vnode, [`kind-${n.kind}`, n.elementType ?? '']);
    }
}

/** A 12×12 square centred on its parent's border, filled by glyph:
 * `in` light, `out` dark, `inout` half and half, `none` neutral. */
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
        const style = p.style ?? fallbackPortStyle(p.direction);
        const glyph = style.glyph ?? p.direction ?? 'none';
        const fill = glyph === 'inout' ? '#fff' : style.fill;
        const vnode = (
            <g class-sysml-port={true} class-selected={selected} data-sysml-ref={p.ref}>
                <rect
                    bboxElement={true}
                    x={0}
                    y={0}
                    width={size}
                    height={size}
                    fill={fill}
                    stroke={selected ? '#1d4ed8' : style.stroke}
                    stroke-width={selected ? 2 : 1.2}
                    stroke-dasharray={unresolved ? '2,2' : undefined}
                />
                {glyph === 'inout' && <polygon points={`0,0 ${size},0 0,${size}`} fill={style.stroke} />}
                {context.renderChildren(port)}
            </g>
        );
        return addClasses(vnode, ['port', glyph]);
    }
}

/** A label child: the name of a node/port, a stereotype or banner line, a
 * compartment line, an edge's keyword/label, or a free IR label. Text is
 * anchored at its top-left — sprotty's `alignFeature` shifts the baseline so
 * `position` (from ELK or the vbox) is the label box's corner. */
@injectable()
export class SysmlLabelView extends ShapeView implements IView {
    render(label: Readonly<SLabelImpl>, _context: RenderingContext, _args?: IViewArgs): VNode | undefined {
        const l = label as Readonly<SysmlLabel>;
        const parent = l.parent as unknown as Partial<WithShapeFields> & {
            type?: string;
            style?: NodeStyle | EdgeStyle;
        };
        const onPort = parent.type === 'port';
        const role = l.role ?? 'name';
        const nodeStyle = parent.type === 'edge' ? undefined : (parent.style as NodeStyle | undefined);
        let fontSize = 12;
        let weight = 'bold';
        let italic = false;
        let fill = nodeStyle?.text ?? '#222';
        let halo = false;
        switch (role) {
            case 'stereotype':
            case 'banner':
                fontSize = 9;
                weight = 'normal';
                italic = true;
                fill = nodeStyle?.stroke ?? '#444';
                break;
            case 'line':
                fontSize = 10;
                weight = 'normal';
                fill = '#333';
                break;
            case 'edge':
            case 'keyword':
                fontSize = 10;
                weight = 'normal';
                italic = role === 'keyword';
                fill = parent.type === 'edge' ? edgeStyleOf(parent as unknown as SysmlEdge).stroke : '#444';
                halo = true;
                break;
            case 'free':
                weight = 'normal';
                break;
            default:
                if (onPort) {
                    fontSize = 9;
                    weight = 'normal';
                }
                italic = !!parent.isAbstract;
        }
        const vnode = (
            <text
                class-sysml-label={true}
                font-size={fontSize}
                font-weight={weight}
                font-style={italic ? 'italic' : undefined}
                font-family="system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif"
                fill={fill}
                stroke={halo ? '#fff' : undefined}
                stroke-width={halo ? 3 : undefined}
                paint-order={halo ? 'stroke' : undefined}
            >
                {l.text}
            </text>
        );
        return addClasses(vnode, [`label-${role}`]);
    }
}

/** A compartment: a separator line across the parent block, then its line
 * labels (stacked by the vbox micro-layout). */
@injectable()
export class SysmlCompartmentView implements IView {
    render(compartment: Readonly<SCompartmentImpl>, context: RenderingContext, _args?: IViewArgs): VNode | undefined {
        const parent = compartment.parent as unknown as { size?: { width: number }; style?: NodeStyle };
        const width = parent.size?.width ?? compartment.size?.width ?? 160;
        const stroke = parent.style?.stroke ?? '#888';
        return (
            <g class-sysml-compartment={true}>
                <line x1={0} y1={0} x2={width} y2={0} stroke={stroke} stroke-width={1} opacity={0.7} />
                {context.renderChildren(compartment)}
            </g>
        );
    }
}

// ---------------------------------------------------------------------------
// Edges
// ---------------------------------------------------------------------------

/** Label class for `label:edge` children: an `SLabelImpl` minus sprotty's
 * `edgeLayoutFeature`, so its ELK-computed `position` is rendered as is. */
export class SysmlEdgeLabelImpl extends SLabelImpl {
    override hasFeature(feature: symbol): boolean {
        return feature !== edgeLayoutFeature && super.hasFeature(feature);
    }
}

@injectable()
export class SysmlEdgeView extends PolylineEdgeView {
    override render(edge: Readonly<SEdgeImpl>, context: RenderingContext, args?: IViewArgs): VNode | undefined {
        const vnode = super.render(edge, context, args);
        if (!vnode) {
            return vnode;
        }
        const e = edge as Readonly<SysmlEdge>;
        vnode.data = vnode.data ?? {};
        vnode.data.attrs = {
            ...(vnode.data.attrs ?? {}),
            'data-sysml-ref': e.ref ?? '',
            'data-sysml-source': e.sourceId,
            'data-sysml-target': e.targetId,
        };
        return addClasses(vnode, ['sysml-edge', `kind-${e.kind ?? 'edge'}`]);
    }

    protected override renderLine(
        edge: Readonly<SEdgeImpl>,
        segments: Point[],
        context: RenderingContext,
        args?: IViewArgs,
    ): VNode {
        const vnode = super.renderLine(edge, segments, context, args);
        const style = edgeStyleOf(edge as Readonly<SysmlEdge>);
        const attrs: Record<string, string | number> = {
            fill: 'none',
            stroke: style.stroke,
            'stroke-width': style.width ?? 1.4,
        };
        if (style.dash) {
            attrs['stroke-dasharray'] = style.dash;
        }
        if (style.arrowTarget && style.arrowTarget !== 'none') {
            attrs['marker-end'] = `url(#${markerId(style.arrowTarget, style.stroke)})`;
        }
        if (style.arrowSource && style.arrowSource !== 'none') {
            attrs['marker-start'] = `url(#${markerId(style.arrowSource, style.stroke)})`;
        }
        vnode.data = vnode.data ?? {};
        vnode.data.attrs = { ...(vnode.data.attrs ?? {}), ...attrs };
        return vnode;
    }

}
