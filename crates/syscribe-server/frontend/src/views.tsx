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
    RoutedPoint,
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
import { ArrowHead, ConfigState, EdgeStyle, FeatureMark, FeatureState, LabelRole, NodeMark, NodeStyle, PortStyle } from './types';
import { safetySymbol } from './safety-shape';

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
type SysmlNode = SNodeImpl &
    WithShapeFields & {
        style?: NodeStyle;
        banners?: string[];
        feature?: FeatureMark;
        mark?: NodeMark;
        analysis?: FeatureState;
        collapsedCount?: number;
        matched?: boolean;
        config?: ConfigState;
    };
type SysmlPort = SPortImpl & WithShapeFields & { style?: PortStyle };
type SysmlLabel = SLabelImpl & { role?: LabelRole };
type SysmlEdge = SEdgeImpl & { kind: string; style?: EdgeStyle; ref?: string; label?: string; overlay?: boolean };

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

/** The SVG of a glyph-kind node, or `undefined` for a box kind. The server
 * sizes these (`vis::size::glyph_size`): initial/final 20×20, fork/join
 * 60×6, decision/merge 28×28. */
function glyphShape(kind: string, width: number, height: number, style: NodeStyle, selected: boolean): VNode | undefined {
    const cx = width / 2;
    const cy = height / 2;
    const outline = selected ? '#1d4ed8' : style.stroke;
    switch (kind) {
        case 'initial': {
            const r = Math.min(width, height) / 2;
            return <circle cx={cx} cy={cy} r={r} fill="#222" stroke={outline} stroke-width={selected ? 2.5 : 0} />;
        }
        case 'final': {
            const r = Math.min(width, height) / 2;
            return (
                <g>
                    <circle cx={cx} cy={cy} r={r} fill="#fff" stroke={outline} stroke-width={selected ? 2.5 : 1.2} />
                    <circle cx={cx} cy={cy} r={r * 0.6} fill="#222" />
                </g>
            );
        }
        case 'fork':
        case 'join':
            return <rect x={0} y={0} width={width} height={height} rx={2} fill={style.fill} stroke={outline} stroke-width={selected ? 1.5 : 0} />;
        case 'decision':
        case 'merge':
            return (
                <path
                    d={`M ${cx},0 L ${width},${cy} L ${cx},${height} L 0,${cy} z`}
                    fill={style.fill}
                    stroke={outline}
                    stroke-width={selected ? 2.5 : 1.4}
                />
            );
        default:
            return undefined;
    }
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

/** The right edge of a node's label stack (a fragment's keyword tab). */
function labelStackRight(node: Readonly<SNodeImpl>): number {
    let right = 0;
    for (const c of node.children) {
        if (c instanceof SLabelImpl) {
            right = Math.max(right, c.bounds.x + c.bounds.width);
        }
    }
    return right;
}

/** The lowest edge of any node of the diagram, in root coordinates — where a
 * lifeline's stem ends (sequence diagrams, `REQ-TRS-VIS-021`). */
function diagramBottom(node: Readonly<SNodeImpl>): number {
    let bottom = 0;
    const walk = (parent: { readonly children: readonly unknown[] }, ay: number): void => {
        for (const c of parent.children) {
            if (c instanceof SNodeImpl) {
                const y = ay + c.bounds.y;
                bottom = Math.max(bottom, y + c.bounds.height);
                walk(c, y);
            }
        }
    };
    walk(node.root, 0);
    return bottom;
}

/** A stick figure with its head's top at `top`, centred on `cx`. */
function stickFigure(cx: number, top: number, style: NodeStyle, dashed: boolean): VNode {
    const headR = 5;
    const hy = top + headR;
    const by = top + 2 * headR;
    const ly = by + 12;
    const d = `M ${cx},${by} L ${cx},${ly} M ${cx - 9},${by + 4} L ${cx + 9},${by + 4} M ${cx},${ly} L ${cx - 7},${ly + 10} M ${cx},${ly} L ${cx + 7},${ly + 10}`;
    return (
        <g class-sysml-actor-figure={true}>
            <circle cx={cx} cy={hy} r={headR} fill={style.fill} stroke={style.stroke} stroke-width={1.4} stroke-dasharray={dashed ? '3,2' : undefined} />
            <path d={d} fill="none" stroke={style.stroke} stroke-width={1.4} stroke-dasharray={dashed ? '3,2' : undefined} />
        </g>
    );
}

// ---------------------------------------------------------------------------
// Feature diagram nodes (`REQ-TRS-FMED-001`, `-002`)
// ---------------------------------------------------------------------------

/** Fill and outline per analysis state: the colour is a second channel, the
 * outline pattern (dead dashed with a strike, false-optional double) the first,
 * so the state survives greyscale printing and colour blindness. */
const FEATURE_STATE: Record<FeatureState, { fill: string; stroke: string }> = {
    normal: { fill: '#ffffff', stroke: '#44546a' },
    core: { fill: '#e6f0fb', stroke: '#1d6fb8' },
    dead: { fill: '#fdecea', stroke: '#b3261e' },
    falseOptional: { fill: '#fff4d6', stroke: '#b7791f' },
};

/** The configurator's fill and badge per state: a chosen feature is solid, an
 * implied one a ring, so what the user decided and what the model decided for
 * them never look alike. */
const CONFIG_STATE: Record<Exclude<ConfigState, 'free'>, { fill: string; badge: string; glyph: string; solid: boolean }> = {
    selected: { fill: '#e3f5e8', badge: '#1e8a3c', glyph: 'M -3.5,0 L -1,3 L 4,-3.5', solid: true },
    forcedOn: { fill: '#f0f9f2', badge: '#1e8a3c', glyph: 'M -3.5,0 L -1,3 L 4,-3.5', solid: false },
    deselected: { fill: '#eeeeee', badge: '#b3261e', glyph: 'M -3.5,-3.5 L 3.5,3.5 M 3.5,-3.5 L -3.5,3.5', solid: true },
    forcedOff: { fill: '#f6f6f6', badge: '#8a8f98', glyph: 'M -3.5,-3.5 L 3.5,3.5 M 3.5,-3.5 L -3.5,3.5', solid: false },
};

/** Radius of the group arc under a feature with an alternative or or group. */
const GROUP_ARC_RADIUS = 24;

type Connected = { kind?: string; target?: { bounds: { x: number; y: number; width: number; height: number } } };

/** The wedge that joins a feature's children into one group: an empty wedge
 * for an alternative (XOR) group, a filled one for an or group. `null` when
 * the feature has fewer than two drawn children. */
function groupArc(node: Readonly<SysmlNode>, width: number, height: number): VNode | undefined {
    const group = node.feature?.group;
    if (group !== 'alternative' && group !== 'or') {
        return undefined;
    }
    const edges = ((node as unknown as { outgoingEdges?: Iterable<Connected> }).outgoingEdges ?? []) as Iterable<Connected>;
    const cx = width / 2;
    const angles: number[] = [];
    for (const e of edges) {
        if (e.kind !== 'child' || !e.target) {
            continue;
        }
        const b = e.target.bounds;
        const dx = b.x + b.width / 2 - (node.bounds.x + cx);
        const dy = b.y - (node.bounds.y + height);
        angles.push(Math.atan2(Math.max(dy, 1), dx));
    }
    if (angles.length < 2) {
        return undefined;
    }
    const lo = Math.min(...angles);
    const hi = Math.max(...angles);
    const r = GROUP_ARC_RADIUS;
    const at = (a: number): string => `${(cx + r * Math.cos(a)).toFixed(1)},${(height + r * Math.sin(a)).toFixed(1)}`;
    const large = hi - lo > Math.PI ? 1 : 0;
    const d = `M ${cx},${height} L ${at(lo)} A ${r} ${r} 0 ${large} 1 ${at(hi)} Z`;
    return <path d={d} fill={group === 'or' ? '#44546a' : 'none'} stroke="#44546a" stroke-width={1.2} class-group-arc={true} />;
}

function featureNodeView(node: Readonly<SysmlNode>, context: RenderingContext, width: number, height: number): VNode {
    const n = node;
    const state = FEATURE_STATE[n.analysis ?? 'normal'];
    const cfg = n.config && n.config !== 'free' ? CONFIG_STATE[n.config] : undefined;
    const selected = !!n.selected;
    const mark = n.feature;
    const incoming = ((n as unknown as { incomingEdges?: Iterable<Connected> }).incomingEdges ?? []) as Iterable<Connected>;
    let hasParent = false;
    for (const e of incoming) {
        if (e.kind === 'child') {
            hasParent = true;
        }
    }
    const outline = selected ? '#1d4ed8' : n.matched ? '#e8590c' : state.stroke;
    const strokeW = selected || n.matched ? 3 : n.analysis && n.analysis !== 'normal' ? 2.2 : 1.5;
    const dashed = n.analysis === 'dead' || !!n.isAbstract;
    const collapsed = (n.collapsedCount ?? 0) > 0;
    const canToggle = (mark?.childCount ?? 0) > 0;
    const vnode = (
        <g
            class-sysml-node={true}
            class-selected={selected}
            class-feature={true}
            class-abstract={!!n.isAbstract}
            data-sysml-ref={n.ref}
            data-feature-state={n.analysis ?? 'normal'}
        >
            <rect x={0} y={0} width={width} height={height} rx={5} fill={cfg ? cfg.fill : state.fill} stroke={outline} stroke-width={strokeW} stroke-dasharray={dashed ? '6,3' : undefined} />
            {n.analysis === 'falseOptional' && (
                <rect x={3} y={3} width={width - 6} height={height - 6} rx={3} fill="none" stroke={outline} stroke-width={1} />
            )}
            {n.analysis === 'dead' && <line x1={4} y1={height - 4} x2={width - 4} y2={4} stroke={outline} stroke-width={1.2} opacity={0.6} />}
            {cfg && (
                <g class-config-badge={true} data-config-state={n.config} transform="translate(12,12)">
                    <circle r={8} fill={cfg.solid ? cfg.badge : '#ffffff'} stroke={cfg.badge} stroke-width={1.6} />
                    <path d={cfg.glyph} fill="none" stroke={cfg.solid ? '#ffffff' : cfg.badge} stroke-width={1.8} stroke-linecap="round" stroke-linejoin="round" />
                </g>
            )}
            {hasParent && mark && (
                <circle cx={width / 2} cy={-7} r={5} fill={mark.mandatory ? '#2b3440' : '#ffffff'} stroke="#2b3440" stroke-width={1.5} class-feature-mark={true} />
            )}
            {groupArc(n, width, height)}
            {canToggle && (
                <g class-fm-toggle={true} data-fm-toggle={n.id} transform={`translate(${width - 9},${height})`}>
                    <circle r={8} fill="#ffffff" stroke="#44546a" stroke-width={1.2} />
                    <path d={collapsed ? 'M -4,0 H 4 M 0,-4 V 4' : 'M -4,0 H 4'} stroke="#44546a" stroke-width={1.6} fill="none" />
                </g>
            )}
            {collapsed && (
                <text x={width - 22} y={height + 4} text-anchor="end" font-size={11} fill="#44546a" class-fm-hidden-count={true}>
                    {`+${n.collapsedCount}`}
                </text>
            )}
            {context.renderChildren(n)}
        </g>
    );
    return addClasses(vnode, ['kind-feature', n.elementType ?? '']);
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
        if (n.kind === 'feature') {
            return featureNodeView(n, context, width, height);
        }
        const container = isContainerKind(n.kind);
        const style = n.style ?? fallbackNodeStyle(n.elementType, n.kind);

        // Fault-tree / attack-tree / GSN symbols (GH #223): the outline from
        // `safety-shape.ts` (the path-for-path mirror of `vis::shape`), the
        // fill and stroke from the resolved style (tone folded in).
        const symbol = safetySymbol(n.kind, width, height);
        if (symbol) {
            const outline = n.selected ? '#1d4ed8' : style.stroke;
            const sw = n.selected ? 2.5 : (style.strokeWidth ?? 1.4);
            const dash = n.resolved === false || style.dashed ? '4,3' : undefined;
            const o = symbol.outline;
            const shape =
                o.type === 'rect' ? (
                    <rect x={0} y={0} width={width} height={height} rx={o.rx} fill={style.fill} stroke={outline} stroke-width={sw} stroke-dasharray={dash} />
                ) : o.type === 'ellipse' ? (
                    <ellipse cx={width / 2} cy={height / 2} rx={width / 2} ry={height / 2} fill={style.fill} stroke={outline} stroke-width={sw} stroke-dasharray={dash} />
                ) : (
                    <path d={o.d} fill={style.fill} stroke={outline} stroke-width={sw} stroke-dasharray={dash} />
                );
            const extras = symbol.extras.map(x =>
                x.type === 'stroke' ? (
                    <path d={x.d} fill="none" stroke={outline} stroke-width={sw} />
                ) : x.type === 'circle' ? (
                    <circle cx={x.cx} cy={x.cy} r={x.r} fill="#fff" stroke={outline} stroke-width={sw} />
                ) : x.type === 'diamond' ? (
                    <path d={x.d} fill="#fff" stroke={outline} stroke-width={sw} />
                ) : (
                    <text x={x.x} y={x.y} font-size={10} font-weight="bold" fill={outline} font-family="Helvetica, Arial, sans-serif">
                        {x.text}
                    </text>
                ),
            );
            const snode = (
                <g class-sysml-node={true} class-selected={!!n.selected} class-unresolved={n.resolved === false} data-sysml-ref={n.ref} data-mark-tone={n.mark?.tone}>
                    {shape}
                    {extras}
                    {context.renderChildren(node)}
                </g>
            );
            return addClasses(snode, [`kind-${n.kind}`, n.elementType ?? '']);
        }
        // `selected` also doubles as the connect-mode "pending source" highlight
        // (`ConnectMouseListener` dispatches a plain `SelectAction`).
        const selected = !!n.selected;
        const unresolved = n.resolved === false;
        const dashed = unresolved || !!style.dashed;
        const outlineWidth = selected ? 2.5 : container ? 1.2 : 1.5;
        const outlineColor = selected ? '#1d4ed8' : style.stroke;
        const header = style.headerFill && !container ? labelStackHeight(n) + 4 : 0;

        // Pseudostate and control-node glyphs (StateMachine / Action kinds,
        // `REQ-TRS-VIS-018`/`-019`): an initial dot, a final bullseye, a
        // fork/join bar, a decision/merge diamond. Their label children (a
        // control node's name, a decision's condition) are placed by ELK
        // beside the glyph (`isGlyphKind` in `layout.ts`).
        const glyph = glyphShape(n.kind, width, height, style, selected);
        if (glyph) {
            const gnode = (
                <g class-sysml-node={true} class-selected={selected} class-unresolved={unresolved} data-sysml-ref={n.ref}>
                    {glyph}
                    {context.renderChildren(node)}
                </g>
            );
            return addClasses(gnode, [`kind-${n.kind}`, n.elementType ?? '']);
        }

        // Sequence kinds (spec §8.16.8.3, `REQ-TRS-VIS-021`), drawn as
        // `vis::svg` draws them: a lifeline is its header box (an actor its
        // stick figure under the name) plus a dashed stem to the diagram's
        // bottom; an activation a bare bar; a fragment an open box with a
        // keyword tab around its label stack.
        if (n.kind === 'lifeline' || n.kind === 'actor') {
            // The header is the carried (server) size: a lifeline that nests
            // its activation is a compound node ELK's `fixed` run grows
            // around the child, but the box drawn is the header alone.
            const carried = (n as unknown as { serverSize?: { width: number; height: number } }).serverSize;
            const headerW = carried?.width ?? width;
            const headerH = carried?.height ?? height;
            const cx = headerW / 2;
            const stemEnd = diagramBottom(n) - n.bounds.y + 12;
            const seq = (
                <g class-sysml-node={true} class-selected={selected} class-unresolved={unresolved} data-sysml-ref={n.ref}>
                    {n.kind === 'lifeline' ? (
                        <rect x={0} y={0} width={headerW} height={headerH} fill={style.fill} stroke={outlineColor} stroke-width={outlineWidth} stroke-dasharray={dashed ? '6,3' : undefined} />
                    ) : (
                        stickFigure(cx, labelStackHeight(n) + 2, style, dashed)
                    )}
                    {stemEnd > headerH && (
                        <line x1={cx} y1={headerH} x2={cx} y2={stemEnd} stroke={outlineColor} stroke-width={1.2} stroke-dasharray="6,4" />
                    )}
                    {context.renderChildren(node)}
                </g>
            );
            return addClasses(seq, [`kind-${n.kind}`, n.elementType ?? '']);
        }
        if (n.kind === 'activation') {
            const seq = (
                <g class-sysml-node={true} class-selected={selected} data-sysml-ref={n.ref}>
                    <rect x={0} y={0} width={width} height={height} fill={style.fill} stroke={outlineColor} stroke-width={selected ? 2 : 1.2} />
                </g>
            );
            return addClasses(seq, [`kind-${n.kind}`]);
        }
        if (n.kind === 'fragment') {
            const tw = Math.min(width, labelStackRight(n) + 8);
            const th = Math.min(height, labelStackHeight(n) + 4);
            const seq = (
                <g class-sysml-node={true} class-selected={selected} class-unresolved={unresolved} data-sysml-ref={n.ref}>
                    <rect x={0} y={0} width={width} height={height} fill="none" stroke={outlineColor} stroke-width={outlineWidth} stroke-dasharray={dashed ? '6,3' : undefined} />
                    {th > 0 && (
                        <path d={`M 0,0 L ${tw},0 L ${tw},${th - 6} L ${tw - 6},${th} L 0,${th} z`} fill={style.fill} stroke={outlineColor} stroke-width={1.2} />
                    )}
                    {context.renderChildren(node)}
                </g>
            );
            return addClasses(seq, [`kind-${n.kind}`]);
        }

        const rounded = container || n.kind === 'state' || n.kind === 'action';
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
                    rx={rounded ? 8 : 4}
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
            case 'value':
                fontSize = 10;
                weight = 'normal';
                fill = '#333';
                break;
            case 'status':
                fontSize = 10;
                weight = 'bold';
                fill = nodeStyle?.stroke ?? '#444';
                break;
            case 'badge':
                fontSize = 9;
                weight = 'normal';
                italic = true;
                fill = nodeStyle?.stroke ?? '#444';
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
                font-family="Helvetica, Arial, 'Liberation Sans', 'DejaVu Sans', sans-serif"
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

/** A cross-tree constraint's curve between two feature boxes: an S from the
 * lower edge of the upper box to the upper edge of the lower one, or, for two
 * boxes on one level, an arc below them whose depth grows with their distance. */
function constraintCurve(
    a: { x: number; y: number; width: number; height: number },
    b: { x: number; y: number; width: number; height: number },
): { d: string; mid: { x: number; y: number } } {
    const ax = a.x + a.width / 2;
    const bx = b.x + b.width / 2;
    const sameLevel = Math.abs(a.y - b.y) < Math.max(a.height, b.height);
    let p0: { x: number; y: number };
    let p3: { x: number; y: number };
    let p1: { x: number; y: number };
    let p2: { x: number; y: number };
    if (sameLevel) {
        const depth = Math.min(34 + Math.abs(bx - ax) * 0.12, 130);
        p0 = { x: ax, y: a.y + a.height };
        p3 = { x: bx, y: b.y + b.height };
        p1 = { x: ax, y: p0.y + depth };
        p2 = { x: bx, y: p3.y + depth };
    } else {
        const [hi, lo, flip] = a.y < b.y ? [a, b, false] : [b, a, true];
        const hx = hi.x + hi.width / 2;
        const lx = lo.x + lo.width / 2;
        const k = Math.max(30, (lo.y - (hi.y + hi.height)) / 2);
        const start = { x: hx, y: hi.y + hi.height };
        const end = { x: lx, y: lo.y };
        const q1 = { x: hx, y: start.y + k };
        const q2 = { x: lx, y: end.y - k };
        [p0, p1, p2, p3] = flip ? [end, q2, q1, start] : [start, q1, q2, end];
    }
    const mid = { x: (p0.x + 3 * p1.x + 3 * p2.x + p3.x) / 8, y: (p0.y + 3 * p1.y + 3 * p2.y + p3.y) / 8 };
    return { d: `M ${p0.x},${p0.y} C ${p1.x},${p1.y} ${p2.x},${p2.y} ${p3.x},${p3.y}`, mid };
}

@injectable()
export class SysmlEdgeView extends PolylineEdgeView {
    override render(edge: Readonly<SEdgeImpl>, context: RenderingContext, args?: IViewArgs): VNode | undefined {
        const e = edge as Readonly<SysmlEdge>;
        const pts = edge.routingPoints;
        let vnode: VNode | undefined;
        if ((e.kind === 'message' || e.kind === 'return') && pts.length >= 2) {
            // A sequence message pinned to its row (`REQ-TRS-VIS-021`) runs
            // along its waypoints alone — stem to stem — never from the header
            // boxes, exactly as `vis::svg::pinned_layout` routes it.
            const route: RoutedPoint[] = pts.map((p, i) => ({
                kind: i === 0 ? 'source' : i === pts.length - 1 ? 'target' : 'linear',
                x: p.x,
                y: p.y,
            }));
            vnode = (
                <g class-sprotty-edge={true} class-mouseover={edge.hoverFeedback}>
                    {this.renderLine(edge, route, context, args)}
                    {this.renderAdditionals(edge, route, context)}
                    {context.renderChildren(edge, { route })}
                </g>
            );
        } else if (e.kind === 'child' || e.overlay) {
            vnode = this.renderFeatureEdge(e, context, args);
        } else {
            vnode = super.render(edge, context, args);
        }
        if (!vnode) {
            return vnode;
        }
        vnode.data = vnode.data ?? {};
        vnode.data.attrs = {
            ...(vnode.data.attrs ?? {}),
            'data-sysml-ref': e.ref ?? '',
            'data-sysml-source': e.sourceId,
            'data-sysml-target': e.targetId,
        };
        return addClasses(vnode, ['sysml-edge', `kind-${e.kind ?? 'edge'}`]);
    }

    /** A feature diagram's edges (`REQ-TRS-FMED-001`): a tree edge is a straight
     * line from the parent's bottom centre to the child's top centre, so the group
     * wedge at the parent meets every child alike; a cross-tree constraint is a
     * curve that leaves and enters the features on their free sides (below a row
     * for two features on one level), so it never runs through a third feature,
     * labelled at its midpoint. */
    private renderFeatureEdge(e: Readonly<SysmlEdge>, context: RenderingContext, args?: IViewArgs): VNode | undefined {
        const src = e.source;
        const tgt = e.target;
        if (!src || !tgt) {
            return undefined;
        }
        const sb = src.bounds;
        const tb = tgt.bounds;
        if (e.kind === 'child') {
            const route: RoutedPoint[] = [
                { kind: 'source', x: sb.x + sb.width / 2, y: sb.y + sb.height },
                { kind: 'target', x: tb.x + tb.width / 2, y: tb.y },
            ];
            return (
                <g class-sprotty-edge={true} class-mouseover={e.hoverFeedback}>
                    {this.renderLine(e, route, context, args)}
                </g>
            );
        }
        const c = constraintCurve(sb, tb);
        const style = edgeStyleOf(e);
        const arrowEnd = style.arrowTarget && style.arrowTarget !== 'none' ? `url(#${markerId(style.arrowTarget, style.stroke)})` : undefined;
        const arrowStart = style.arrowSource && style.arrowSource !== 'none' ? `url(#${markerId(style.arrowSource, style.stroke)})` : undefined;
        const keyword = e.style?.keyword;
        return (
            <g class-sprotty-edge={true} class-mouseover={e.hoverFeedback}>
                <path d={c.d} fill="none" stroke={style.stroke} stroke-width={style.width ?? 1.4} stroke-dasharray={style.dash ?? undefined} marker-end={arrowEnd} marker-start={arrowStart} />
                {keyword && (
                    <text x={c.mid.x} y={c.mid.y - 4} text-anchor="middle" font-size={10} font-style="italic" fill={style.stroke} class-fm-edge-keyword={true}>
                        {keyword}
                    </text>
                )}
            </g>
        );
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
