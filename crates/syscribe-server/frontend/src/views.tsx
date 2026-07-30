/** @jsx svg */
// Minimal node/edge views for the SysML block-style diagrams
// (`ADR-SYS-DE-001`). Colors intentionally mirror
// `crates/syscribe-model/src/renderer.rs`'s `render_shape`/`edge_style` so the
// sprotty editor and the legacy read-only SVG renderer (still used for
// `Mermaid`-kind diagrams and anywhere `render_diagram` is called directly)
// look like the same visual language rather than two unrelated tools.
import { injectable } from 'inversify';
import { VNode } from 'snabbdom';
import {
    IView,
    IViewArgs,
    PolylineEdgeView,
    RenderingContext,
    SEdgeImpl,
    ShapeView,
    SNodeImpl,
    svg,
} from 'sprotty';
import { Point } from 'sprotty-protocol';

/** Extra fields sprotty's `SModelFactory` copies onto the instance verbatim
 * from `SysmlNodeSchema`/`SysmlEdgeSchema` (see `types.ts`) — not part of
 * `SNodeImpl`/`SEdgeImpl` themselves, so views read them through this cast. */
type WithSysmlNodeFields = SNodeImpl & { kind: string; name: string; ref: string; isAbstract?: boolean };
type WithSysmlEdgeFields = SEdgeImpl & { kind: string };

interface KindStyle {
    fill: string;
    stroke: string;
    headerFill?: string;
    stereotype: string;
}

function nodeStyle(kind: string): KindStyle {
    switch (kind) {
        case 'RequirementDef':
            return { fill: '#f9f7ff', stroke: '#4a0a6e', headerFill: '#4a0a6e', stereotype: 'requirement def' };
        case 'Requirement':
            return { fill: '#f9f7ff', stroke: '#4a0a6e', headerFill: '#4a0a6e', stereotype: 'requirement' };
        case 'TestCase':
        case 'TestCaseDef':
            return { fill: '#f0fff4', stroke: '#1e6b2e', headerFill: '#1e6b2e', stereotype: 'test case' };
        case 'PartDef':
            return { fill: '#f5f5fa', stroke: '#3a3a4a', stereotype: 'part def' };
        case 'Part':
            return { fill: '#f5f5fa', stroke: '#3a3a4a', stereotype: 'part' };
        default:
            return { fill: '#f5f5fa', stroke: '#666', stereotype: kind };
    }
}

function edgeStyle(kind: string): { stroke: string; dash?: string; label: string } {
    switch (kind) {
        case 'derivedFrom':
            return { stroke: '#555', dash: '5,3', label: 'derived from' };
        case 'verifies':
            return { stroke: '#3a6ea5', label: 'verifies' };
        case 'allocatedTo':
            return { stroke: '#7a3ea5', dash: '3,3', label: 'allocated to' };
        default:
            return { stroke: '#888', label: kind };
    }
}

@injectable()
export class SysmlNodeView extends ShapeView implements IView {
    render(node: Readonly<SNodeImpl>, _context: RenderingContext, _args?: IViewArgs): VNode | undefined {
        const n = node as Readonly<WithSysmlNodeFields>;
        const width = n.size?.width ?? 200;
        const height = n.size?.height ?? 50;
        const style = nodeStyle(n.kind);
        // `selected` also doubles as the connect-mode "pending source" highlight
        // (`ConnectMouseListener` dispatches a plain `SelectAction`) — one less
        // bespoke visual state to wire up for a gesture that's inherently
        // transient (cleared as soon as the second node is clicked).
        const selected = !!n.selected;
        const outlineWidth = selected ? 2.5 : 1.5;
        const outlineColor = selected ? '#1d4ed8' : style.stroke;

        return (
            <g class-sysml-node={true} class-selected={selected}>
                <rect
                    x={0} y={0} width={width} height={height} rx={4}
                    fill={style.fill} stroke={outlineColor} stroke-width={outlineWidth}
                />
                {style.headerFill && (
                    <rect x={0} y={0} width={width} height={18} rx={4} fill={style.headerFill} opacity={0.12} />
                )}
                <text x={width / 2} y={13} text-anchor="middle" font-size={9} fill={style.stroke} font-style="italic">
                    &#171;{style.stereotype}&#187;
                </text>
                <text x={width / 2} y={height / 2 + 12} text-anchor="middle" font-size={12} font-weight="bold" fill="#222">
                    {n.name}
                </text>
                {n.isAbstract && (
                    <text x={width / 2} y={height - 4} text-anchor="middle" font-size={9} fill="#666" font-style="italic">
                        isAbstract
                    </text>
                )}
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
