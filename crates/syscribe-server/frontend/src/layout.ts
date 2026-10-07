// ELK layout in the browser (`REQ-TRS-VIS-007`, design §6.2). Three pieces:
//
// 1. `prepareForLayout` — gives the nested schema from `GET /api/diagrams/model`
//    the sprotty *micro-layout* it needs for the hidden measuring pass
//    (`needsClientLayout: true`): every node and compartment becomes a `vbox`
//    that stacks its label children, the stereotype and `banners` become
//    label children next to the name label, a compartment's `lines` are
//    expanded into label children, and ports get a 12×12 size unless the
//    server sent one. After that pass every element has a real `size`, which
//    is what ELK sizes nodes from.
// 2. `SyscribeLayoutConfigurator` — maps the root's `layoutOptions` (the ELK
//    option ids `vis::sprotty` derives from the IR's layout hints) plus
//    sensible spacing/sizing defaults onto the ELK graph, honours pins by
//    switching ELK into interactive mode (or the `fixed` algorithm when every
//    node is pinned), and fixes port sides from `side`.
// 3. `SyscribeLayoutProcessor` — the `ILayoutPreprocessor`/`ILayoutPostprocessor`
//    pair: before ELK runs, edges of the kinds in `syscribe.reversedEdgeKinds`
//    (inheritance: supertype above subtype) are flipped so ELK layers them
//    upward and every carried server size is stamped onto its ELK shape;
//    afterwards flipped sections are flipped back, every edge section is
//    translated from its ELK `container`'s coordinate system into the root's
//    (sprotty keeps all edges at the root), and in `fixed` mode — where ELK
//    places neither node labels nor port labels — those labels are placed
//    here.
//
// Server sizes are authoritative (`REQ-TRS-VIS-017`). An element that arrives
// with a `size` — every node, port, compartment and label from a server that
// computes them in Rust from the shared text metrics, or just a pin's `w`/`h`
// from an older one — keeps it: `prepareForLayout` copies it to `serverSize`,
// and from then on
//
//   - the hidden measuring pass does not replace it: `container.ts`'s bounds
//     updater reports the carried width/height instead of the `getBBox()`
//     ones (keeping only the measured text offset, which sprotty needs to
//     align a label's baseline), and the element's `vbox` runs with
//     `resizeContainer: false`, so it positions the children — name,
//     stereotype, banners, compartments and their lines — inside the carried
//     size instead of growing to fit them;
//   - ELK lays it out with exactly that width/height: the preprocessor stamps
//     it onto the ELK node/port/label, a sized leaf node's size constraints
//     drop `NODE_LABELS`/`PORT_LABELS` (the server already sized for its
//     text) and keep `PORTS MINIMUM_SIZE` with the carried size as the
//     minimum, so the only thing ELK may ever add is room for ports that
//     physically do not fit on a side; a sized compound node uses it as its
//     minimum, since its real extent comes from its children;
//   - the views draw it, because the schema's `size` is never overwritten by
//     anything else.
//
// An element without a `size` (an older server, a shape `editor.ts` created
// locally, an edge's keyword/label children) is measured in the DOM and sized
// by ELK as before, so the client works against both server versions.
//
// Nothing here writes to the model: an ELK result lives only in the cached
// schema until the user drags (one pin), presses *Pin all* or *Auto-layout*
// (`editor.ts`).

import type { ElkEdgeSection, ElkExtendedEdge, ElkNode, ElkPort, ElkShape, LayoutOptions } from 'elkjs/lib/elk-api';
import type { ILayoutConfigurator, ILayoutPostprocessor, ILayoutPreprocessor } from 'sprotty-elk/lib/elk-layout';
import { SEdge, SGraph, SModelElement, SNode, SPort } from 'sprotty-protocol';
import { SModelIndex } from 'sprotty-protocol/lib/utils/model-utils';
import {
    childrenOf,
    DiagramModelSchema,
    isCompartmentSchema,
    isEdgeSchema,
    isLabelSchema,
    isNodeSchema,
    isPortSchema,
    LabelRole,
    ServerSize,
    SysmlCompartmentSchema,
    SysmlEdgeSchema,
    SysmlLabelSchema,
    SysmlNodeChildSchema,
    SysmlNodeSchema,
    SysmlPortSchema,
} from './types';

export const PORT_SIZE = 12;

/** Anything that may carry a server size: a node, port, label or compartment
 * schema, or the sprotty model element built from one. */
export interface SizedSchema {
    size?: { width: number; height: number };
    serverSize?: ServerSize;
}

/** Copy a positive `size` the server sent to `serverSize` (once; later runs
 * over the same cached schema keep the first copy, since `size` then holds
 * what ELK produced from it). */
export function adoptServerSize(el: SizedSchema): void {
    if (!el.serverSize && el.size && el.size.width > 0 && el.size.height > 0) {
        el.serverSize = { width: el.size.width, height: el.size.height };
    }
}

/** The ELK width/height the carried size dictates, when there is one. */
export function serverSizeOf(el: SizedSchema | undefined): ServerSize | undefined {
    const s = el?.serverSize;
    return s && s.width > 0 && s.height > 0 ? s : undefined;
}

/** Horizontal gap between siblings in a layer, and between layers. */
const NODE_NODE_SPACING = 40;
const LAYER_SPACING = 60;
/** Inner padding of a compound node (boundary) around its children, below
 * whatever its label stack needs. */
const COMPOUND_PADDING = 20;
const MIN_NODE_WIDTH = 120;
const MIN_NODE_HEIGHT = 40;

export function isContainerKind(kind: string | undefined): boolean {
    return kind === 'boundary' || kind === 'system-boundary' || kind === 'swimlane' || kind === 'fragment';
}

/** A placeholder size for a shape created locally before the next measuring
 * pass (`DiagramEditor.addNode`); the server reload replaces it. */
export function defaultSize(kind: string): { width: number; height: number } {
    switch (kind) {
        case 'port':
            return { width: PORT_SIZE, height: PORT_SIZE };
        case 'boundary':
        case 'system-boundary':
        case 'swimlane':
        case 'fragment':
            return { width: 400, height: 260 };
        default:
            return { width: 160, height: 50 };
    }
}

// ---------------------------------------------------------------------------
// 1. Micro-layout preparation
// ---------------------------------------------------------------------------

/** sprotty `vbox` options for a node: labels/compartments stack from the top
 * with no horizontal padding (ELK re-centres the labels; compartments span
 * the node). `resizeContainer` is set per element: `false` when the server
 * sent its size (the children are laid out inside it), `true` otherwise (the
 * container grows to fit the measured children). */
const NODE_VBOX = { paddingTop: 4, paddingBottom: 4, paddingLeft: 0, paddingRight: 0, vGap: 1, hAlign: 'left' as const };
const COMPARTMENT_VBOX = { paddingTop: 4, paddingBottom: 4, paddingLeft: 8, paddingRight: 8, vGap: 2, hAlign: 'left' as const };

function makeLabel(id: string, text: string, role: LabelRole, type: 'label' | 'label:edge' = 'label'): SysmlLabelSchema {
    return { id, type, text, role };
}

function prepareCompartment(c: SysmlCompartmentSchema): void {
    adoptServerSize(c);
    c.layout = 'vbox';
    c.layoutOptions = { ...COMPARTMENT_VBOX, resizeContainer: !c.serverSize };
    const existing = new Set((c.children ?? []).map(k => k.id));
    const lines = c.lines ?? [];
    const kids: SysmlNodeChildSchema[] = [...(c.children ?? [])];
    lines.forEach((line, i) => {
        const id = `${c.id}-line-${i}`;
        if (!existing.has(id)) {
            kids.push(makeLabel(id, line, 'line'));
        }
    });
    c.children = kids;
    for (const k of kids) {
        if (isLabelSchema(k)) {
            adoptServerSize(k);
            if (k.role === undefined) {
                k.role = 'line';
            }
        }
    }
}

function preparePort(p: SysmlPortSchema): void {
    adoptServerSize(p);
    if (!p.serverSize) {
        p.size = { width: PORT_SIZE, height: PORT_SIZE };
    }
    for (const k of childrenOf(p)) {
        if (isLabelSchema(k)) {
            adoptServerSize(k);
            if (k.role === undefined) {
                k.role = 'name';
            }
        }
    }
}

/** Give one node its micro-layout: stereotype/banner labels before the name
 * label, `vbox` on the node and its compartments, 12×12 ports unless sized
 * by the server. Idempotent. */
export function prepareNode(n: SysmlNodeSchema): void {
    adoptServerSize(n);
    n.layout = 'vbox';
    n.layoutOptions = { ...NODE_VBOX, resizeContainer: !n.serverSize };

    const kids: SysmlNodeChildSchema[] = [...(n.children ?? [])];
    const existing = new Set(kids.map(k => k.id));
    const extra: SysmlNodeChildSchema[] = [];
    if (n.stereotype && !existing.has(`${n.id}-stereotype`)) {
        extra.push(makeLabel(`${n.id}-stereotype`, `«${n.stereotype}»`, 'stereotype'));
    }
    (n.banners ?? []).forEach((b, i) => {
        const id = `${n.id}-banner-${i}`;
        if (!existing.has(id)) {
            extra.push(makeLabel(id, `«${b}»`, 'banner'));
        }
    });
    // The name label is the server's first child; keep the stereotype stack
    // above it so both the hidden vbox and ELK's label stacking read top-down.
    n.children = [...extra, ...kids];

    for (const k of n.children) {
        if (isLabelSchema(k)) {
            adoptServerSize(k);
            if (k.role === undefined) {
                k.role = k.id === `${n.id}-label` ? 'name' : 'free';
            }
        } else if (isCompartmentSchema(k)) {
            prepareCompartment(k);
        } else if (isPortSchema(k)) {
            preparePort(k);
        } else if (isNodeSchema(k)) {
            prepareNode(k);
        }
    }
}

/** An edge's `«keyword»` and `label` become label children so ELK reserves
 * room for them along the route (and sprotty measures them). Idempotent. */
export function prepareEdge(e: SysmlEdgeSchema): void {
    const kids: SysmlLabelSchema[] = [...((e.children ?? []) as SysmlLabelSchema[])];
    kids.forEach(adoptServerSize);
    const existing = new Set(kids.map(k => k.id));
    const keyword = e.style?.keyword;
    if (keyword && !existing.has(`${e.id}-keyword`)) {
        kids.push(makeLabel(`${e.id}-keyword`, keyword === '=' ? '=' : `«${keyword}»`, 'keyword', 'label:edge'));
    }
    if (e.label && !existing.has(`${e.id}-label`)) {
        kids.push(makeLabel(`${e.id}-label`, e.label, 'edge', 'label:edge'));
    }
    if (kids.length > 0) {
        e.children = kids;
    }
}

/** Prepare every shape of a freshly fetched model (see the module doc). */
export function prepareForLayout(model: DiagramModelSchema): void {
    for (const child of model.children) {
        if (isEdgeSchema(child)) {
            prepareEdge(child);
        } else if (isNodeSchema(child)) {
            prepareNode(child);
        } else if (isPortSchema(child)) {
            preparePort(child);
        } else if (isCompartmentSchema(child)) {
            prepareCompartment(child);
        } else if (isLabelSchema(child)) {
            adoptServerSize(child);
            if (child.role === undefined) {
                child.role = 'free';
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Layout configurator
// ---------------------------------------------------------------------------

/** Per-run facts the configurator derives from the root and the processor
 * reads back — one instance is shared by both (see `container.ts`). */
export class LayoutState {
    reversedKinds = new Set<string>();
    pinned = new Set<string>();
    anyPinned = false;
    allPinned = false;
    direction = 'DOWN';
    /** Ids of the ELK edges the preprocessor flipped, for the postprocessor. */
    flippedEdges = new Set<string>();

    reset(model: DiagramModelSchema): void {
        const opts = model.layoutOptions ?? {};
        const reversed = opts['syscribe.reversedEdgeKinds'];
        this.reversedKinds = new Set(Array.isArray(reversed) ? reversed : []);
        this.pinned = new Set(model.pinned ?? []);
        this.direction = String(opts['elk.direction'] ?? 'DOWN');
        const nodeIds = collectNodeIds(model);
        this.anyPinned = nodeIds.some(id => this.pinned.has(id));
        this.allPinned = nodeIds.length > 0 && nodeIds.every(id => this.pinned.has(id));
        this.flippedEdges.clear();
    }
}

function collectNodeIds(model: DiagramModelSchema): string[] {
    const ids: string[] = [];
    const walk = (parent: { children?: unknown[] }): void => {
        for (const c of (parent.children ?? []) as SysmlNodeChildSchema[]) {
            if (isNodeSchema(c)) {
                ids.push(c.id);
                walk(c);
            }
        }
    };
    walk(model);
    return ids;
}

function basicType(e: SModelElement): string {
    const idx = e.type.indexOf(':');
    return idx >= 0 ? e.type.slice(0, idx) : e.type;
}

/** ELK's interactive options: pinned positions drive layering and ordering,
 * so unpinned nodes are placed around them (design §6.2). They must be set
 * on every compound node too — ELK refuses a hierarchy whose children use a
 * different crossing-minimisation strategy than the root. */
const INTERACTIVE_OPTIONS: LayoutOptions = {
    'elk.interactive': 'true',
    'elk.layered.layering.strategy': 'INTERACTIVE',
    'elk.layered.crossingMinimization.strategy': 'INTERACTIVE',
    'elk.layered.considerModelOrder.strategy': 'NODES_AND_EDGES',
};

function fallbackSide(port: SysmlPortSchema, direction: string): string {
    const right = direction === 'RIGHT';
    switch (port.direction) {
        case 'in':
            return right ? 'WEST' : 'NORTH';
        case 'out':
            return right ? 'EAST' : 'SOUTH';
        default:
            return right ? 'EAST' : 'SOUTH';
    }
}

export class SyscribeLayoutConfigurator implements ILayoutConfigurator {
    constructor(private readonly state: LayoutState) {}

    apply(element: SModelElement, index: SModelIndex): LayoutOptions | undefined {
        switch (basicType(element)) {
            case 'graph':
                return this.graphOptions(element as SGraph as DiagramModelSchema);
            case 'node':
                return this.nodeOptions(element as SNode as SysmlNodeSchema);
            case 'port':
                return this.portOptions(element as SPort as SysmlPortSchema, index);
            default:
                return undefined;
        }
    }

    protected graphOptions(graph: DiagramModelSchema): LayoutOptions {
        this.state.reset(graph);
        const src = graph.layoutOptions ?? {};
        const str = (k: string): string | undefined => {
            const v = src[k];
            return typeof v === 'string' ? v : undefined;
        };
        const opts: LayoutOptions = {
            'elk.algorithm': this.state.allPinned ? 'fixed' : (str('elk.algorithm') ?? 'layered'),
            'elk.direction': str('elk.direction') ?? 'DOWN',
            'elk.edgeRouting': 'ORTHOGONAL',
            'elk.spacing.nodeNode': String(NODE_NODE_SPACING),
            'elk.layered.spacing.nodeNodeBetweenLayers': String(LAYER_SPACING),
            'elk.spacing.edgeNode': '30',
            'elk.layered.spacing.edgeNodeBetweenLayers': '30',
            'elk.spacing.portPort': '16',
            'elk.spacing.labelLabel': '1',
            'elk.spacing.labelPortHorizontal': '4',
            'elk.spacing.labelPortVertical': '2',
            'elk.padding': '[top=20,left=20,bottom=20,right=20]',
        };
        const hierarchy = str('elk.hierarchyHandling');
        if (hierarchy) {
            opts['elk.hierarchyHandling'] = hierarchy;
        }
        if (this.state.anyPinned && !this.state.allPinned) {
            Object.assign(opts, INTERACTIVE_OPTIONS);
        }
        return opts;
    }

    protected nodeOptions(node: SysmlNodeSchema): LayoutOptions {
        const kids = childrenOf(node) as SysmlNodeChildSchema[];
        const compound = kids.some(isNodeSchema);
        const ports = kids.filter(isPortSchema);
        const server = serverSizeOf(node);
        // A sized leaf node is exactly its server size (ELK only ever adds
        // room for ports that do not fit); a sized compound node is at least
        // that big; an unsized leaf is at least its measured size.
        const measured = node.size ?? { width: 0, height: 0 };
        const own = server ?? (compound ? { width: 0, height: 0 } : measured);
        const minW = server && !compound ? server.width : Math.ceil(Math.max(MIN_NODE_WIDTH, own.width));
        const minH = server && !compound ? server.height : Math.ceil(Math.max(MIN_NODE_HEIGHT, own.height));
        const anySide = ports.some(p => !!p.side);
        const container = isContainerKind(node.kind);

        const opts: LayoutOptions = {
            'elk.nodeSize.constraints': server && !compound ? 'PORTS MINIMUM_SIZE' : 'NODE_LABELS PORTS PORT_LABELS MINIMUM_SIZE',
            'elk.nodeSize.minimum': `(${minW}, ${minH})`,
            'elk.nodeLabels.placement': container ? '[H_LEFT, V_TOP, INSIDE]' : '[H_CENTER, V_TOP, INSIDE]',
            'elk.nodeLabels.padding': '[top=4,left=8,bottom=4,right=8]',
            'elk.portLabels.placement': 'OUTSIDE',
            'elk.portConstraints': anySide ? 'FIXED_SIDE' : 'FREE',
        };
        if (compound) {
            // ELK adds the inside label area to this padding itself.
            const p = COMPOUND_PADDING;
            opts['elk.padding'] = `[top=${p},left=${p},bottom=${p},right=${p}]`;
            if (this.state.allPinned) {
                opts['elk.algorithm'] = 'fixed';
            } else if (this.state.anyPinned) {
                Object.assign(opts, INTERACTIVE_OPTIONS);
            }
        }
        if (this.state.pinned.has(node.id) && node.position) {
            opts['elk.position'] = `(${node.position.x}, ${node.position.y})`;
        }
        return opts;
    }

    protected portOptions(port: SysmlPortSchema, index: SModelIndex): LayoutOptions {
        // Centre the square on the border (ELK's default puts it just
        // outside, touching the border).
        const extent = serverSizeOf(port)?.width ?? port.size?.width ?? PORT_SIZE;
        const opts: LayoutOptions = { 'elk.port.borderOffset': String(-extent / 2) };
        const parent = index.getParent(port.id) as SysmlNodeSchema | undefined;
        const siblings = parent ? (childrenOf(parent) as SysmlNodeChildSchema[]).filter(isPortSchema) : [];
        const parentFixed = siblings.some(p => !!p.side);
        if (port.side) {
            opts['elk.port.side'] = port.side.toUpperCase();
        } else if (parentFixed) {
            opts['elk.port.side'] = fallbackSide(port, this.state.direction);
        }
        return opts;
    }
}

// ---------------------------------------------------------------------------
// 3. Pre/post-processing
// ---------------------------------------------------------------------------

function reverseSection(s: ElkEdgeSection): ElkEdgeSection {
    return {
        ...s,
        startPoint: s.endPoint,
        endPoint: s.startPoint,
        bendPoints: s.bendPoints ? [...s.bendPoints].reverse() : undefined,
        incomingShape: s.outgoingShape,
        outgoingShape: s.incomingShape,
    };
}

function* walkElkNodes(node: ElkNode, ax = 0, ay = 0): Generator<{ node: ElkNode; ax: number; ay: number }> {
    yield { node, ax, ay };
    for (const c of node.children ?? []) {
        yield* walkElkNodes(c, ax + (c.x ?? 0), ay + (c.y ?? 0));
    }
}

/** Which side of its parent a laid-out port sits on, from its centre. */
function sideOf(port: ElkPort, parent: ElkNode): 'north' | 'east' | 'south' | 'west' {
    const w = parent.width ?? 0;
    const h = parent.height ?? 0;
    const cx = (port.x ?? 0) + (port.width ?? 0) / 2;
    const cy = (port.y ?? 0) + (port.height ?? 0) / 2;
    const dist = [
        { side: 'west' as const, d: Math.abs(cx) },
        { side: 'east' as const, d: Math.abs(cx - w) },
        { side: 'north' as const, d: Math.abs(cy) },
        { side: 'south' as const, d: Math.abs(cy - h) },
    ];
    dist.sort((a, b) => a.d - b.d);
    return dist[0].side;
}

/** Place port labels outside the parent next to their port, node labels
 * stacked at the top, and edge labels at the midpoint of the straight line
 * between the edge's ends — what ELK's `fixed` algorithm leaves undone. */
function placeLabelsFixed(root: ElkNode): void {
    const centres = new Map<string, { x: number; y: number }>();
    for (const { node, ax, ay } of walkElkNodes(root)) {
        centres.set(node.id, { x: ax + (node.width ?? 0) / 2, y: ay + (node.height ?? 0) / 2 });
        for (const p of node.ports ?? []) {
            centres.set(p.id, { x: ax + (p.x ?? 0) + (p.width ?? 0) / 2, y: ay + (p.y ?? 0) + (p.height ?? 0) / 2 });
        }
    }
    for (const { edge } of walkElkEdges(root)) {
        if (edge.sections && edge.sections.length > 0) {
            continue;
        }
        const a = centres.get(edge.sources[0]);
        const b = centres.get(edge.targets[0]);
        if (!a || !b) {
            continue;
        }
        const labels = edge.labels ?? [];
        const stack = labels.reduce((h, l) => h + (l.height ?? 0) + 1, 0);
        let y = (a.y + b.y) / 2 - stack - 3;
        for (const l of labels) {
            l.x = (a.x + b.x) / 2 - (l.width ?? 0) / 2;
            l.y = y;
            y += (l.height ?? 0) + 1;
        }
    }
    for (const { node } of walkElkNodes(root)) {
        if (node === root) {
            continue;
        }
        const compound = (node.children ?? []).length > 0;
        let y = 4;
        for (const l of node.labels ?? []) {
            const lw = l.width ?? 0;
            l.x = compound ? 8 : Math.max(0, ((node.width ?? 0) - lw) / 2);
            l.y = y;
            y += (l.height ?? 0) + 1;
        }
        for (const p of node.ports ?? []) {
            const pw = p.width ?? 0;
            const ph = p.height ?? 0;
            for (const l of p.labels ?? []) {
                const lw = l.width ?? 0;
                const lh = l.height ?? 0;
                // Below-and-beside for east/west ports, as ELK's own OUTSIDE
                // placement does, so the label clears the edge line.
                switch (sideOf(p, node)) {
                    case 'west':
                        l.x = -lw - 1;
                        l.y = ph + 1;
                        break;
                    case 'east':
                        l.x = pw + 1;
                        l.y = ph + 1;
                        break;
                    case 'north':
                        l.x = (pw - lw) / 2;
                        l.y = -lh - 2;
                        break;
                    default:
                        l.x = (pw - lw) / 2;
                        l.y = ph + 2;
                }
            }
        }
    }
}

function* walkElkEdges(node: ElkNode): Generator<{ edge: ElkExtendedEdge; owner: ElkNode }> {
    for (const e of node.edges ?? []) {
        yield { edge: e as ElkExtendedEdge, owner: node };
    }
    for (const c of node.children ?? []) {
        yield* walkElkEdges(c);
    }
}

export class SyscribeLayoutProcessor implements ILayoutPreprocessor, ILayoutPostprocessor {
    constructor(private readonly state: LayoutState) {}

    preprocess(elkGraph: ElkNode, _sgraph: SGraph, index: SModelIndex): void {
        this.state.flippedEdges.clear();
        for (const { edge } of walkElkEdges(elkGraph)) {
            const sedge = index.getById(edge.id) as (SEdge & { kind?: string }) | undefined;
            if (sedge && sedge.kind && this.state.reversedKinds.has(sedge.kind)) {
                const sources = edge.sources;
                edge.sources = edge.targets;
                edge.targets = sources;
                this.state.flippedEdges.add(edge.id);
            }
        }
        // Server sizes are authoritative (module doc): stamp each carried
        // size onto its ELK shape — nodes, ports, node/port/edge labels — so
        // ELK lays out with exactly it whatever the hidden pass measured.
        const stamp = (shape: ElkShape): void => {
            const s = shape.id ? serverSizeOf(index.getById(shape.id) as SizedSchema | undefined) : undefined;
            if (s) {
                shape.width = s.width;
                shape.height = s.height;
            }
        };
        for (const { node } of walkElkNodes(elkGraph)) {
            if (node !== elkGraph) {
                stamp(node);
            }
            node.labels?.forEach(stamp);
            for (const p of node.ports ?? []) {
                stamp(p);
                p.labels?.forEach(stamp);
            }
        }
        for (const { edge } of walkElkEdges(elkGraph)) {
            edge.labels?.forEach(stamp);
        }
    }

    postprocess(elkGraph: ElkNode, _sgraph: SGraph, _index: SModelIndex): void {
        const offsets = new Map<string, { x: number; y: number }>();
        for (const { node, ax, ay } of walkElkNodes(elkGraph)) {
            offsets.set(node.id, { x: ax, y: ay });
        }
        for (const { edge, owner } of walkElkEdges(elkGraph)) {
            const e = edge as ElkExtendedEdge & { container?: string };
            if (this.state.flippedEdges.has(e.id)) {
                const sources = e.sources;
                e.sources = e.targets;
                e.targets = sources;
                if (e.sections) {
                    e.sections = [...e.sections].reverse().map(reverseSection);
                }
            }
            // ELK reports an edge's sections relative to its `container`
            // (the lowest common ancestor of its ends); sprotty keeps every
            // edge at the root, so translate into root coordinates.
            const off = offsets.get(e.container ?? owner.id) ?? { x: 0, y: 0 };
            if (off.x !== 0 || off.y !== 0) {
                for (const s of e.sections ?? []) {
                    s.startPoint = { x: s.startPoint.x + off.x, y: s.startPoint.y + off.y };
                    s.endPoint = { x: s.endPoint.x + off.x, y: s.endPoint.y + off.y };
                    s.bendPoints = s.bendPoints?.map(p => ({ x: p.x + off.x, y: p.y + off.y }));
                }
                for (const l of e.labels ?? []) {
                    if (l.x !== undefined && l.y !== undefined) {
                        l.x += off.x;
                        l.y += off.y;
                    }
                }
            }
        }
        if (this.state.allPinned) {
            placeLabelsFixed(elkGraph);
        }
    }
}
