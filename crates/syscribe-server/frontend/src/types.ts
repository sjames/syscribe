// Wire types shared between `crates/syscribe-server/src/routes/diagram_model.rs`
// (the `GET /api/diagrams/model/{*qname}` JSON endpoint — its contract is the
// module doc of `crates/syscribe-model/src/vis/sprotty.rs`, REQ-TRS-VIS-006)
// and `crates/syscribe-server/src/routes/mutate.rs` (the guarded-write
// mutation endpoints) on one side, and the sprotty client on the other.
//
// The graph is **nested**: a node's ports, compartments, name label and
// nested blocks live in its `children`; edges are always root children and
// reference nodes or ports by id across the whole tree. `position` is present
// only for pinned nodes (parent-relative when nested). `size` is the server's
// own measurement (`REQ-TRS-VIS-017`: computed in Rust from shared text
// metrics, on every node, port, compartment and label — or, from an older
// server, only a pin's `w`/`h`); `layout.ts` copies it to `serverSize` and
// treats it as authoritative, measuring in the DOM only what arrived without
// one. Positions are placed by ELK (`layout.ts`).

import { SCompartment, SEdge, SGraph, SLabel, SNode, SPort } from 'sprotty-protocol';

/** A width/height pair as the server sends it (`vis::sprotty`'s `Size`). */
export interface ServerSize {
    width: number;
    height: number;
}

// ---------------------------------------------------------------------------
// Resolved style (`vis::style`, REQ-TRS-VIS-012) — carried per node, port and
// edge so the views read it instead of holding their own colour table. Every
// field is optional on the wire so an older server (or a hand-made schema in
// `editor.ts`) still renders through the fallback tables in `views.tsx`.
// ---------------------------------------------------------------------------

export interface NodeStyle {
    fill: string;
    stroke: string;
    headerFill?: string | null;
    text?: string;
    dashed?: boolean;
}

export type PortGlyph = 'in' | 'out' | 'inout' | 'none';

export interface PortStyle {
    fill: string;
    stroke: string;
    glyph?: PortGlyph;
}

export type ArrowHead =
    | 'none'
    | 'filled'
    | 'open'
    | 'hollowTriangle'
    | 'filledDiamond'
    | 'hollowDiamond'
    | 'filledCircle';

export interface EdgeStyle {
    stroke: string;
    dash?: string | null;
    width?: number;
    arrowTarget?: ArrowHead;
    arrowSource?: ArrowHead;
    keyword?: string | null;
}

/** Fields every IR-backed shape (`node` and `port`) carries. */
interface SysmlShapeFields {
    /** The model reference as authored (`shapes.<id>.ref`), kept even when unresolved. */
    ref: string;
    /** Whether `ref` resolves to a model element; `false` is drawn dashed. */
    resolved?: boolean;
    /** IR node role (`NodeKind::as_str`): `block`, `boundary`, `port`, `state`, `requirement`, … */
    kind: string;
    /** Resolved element type name (`PartDef`, `Requirement`, …), when known. */
    elementType?: string;
    /** SysMLv2 stereotype text without guillemets (`part def`, `port`, …). */
    stereotype?: string;
    /** Display label (also carried by the `<id>-label` child). */
    name: string;
    isAbstract?: boolean;
    /** Ports only: `in` | `out` | `inout`. */
    direction?: string;
    /** Ports only, when fixed: `north` | `east` | `south` | `west`. */
    side?: string;
}

/** A container/block-like IR node (boundary, block, state, requirement, …). */
export interface SysmlNodeSchema extends SNode, SysmlShapeFields {
    type: 'node';
    style?: NodeStyle;
    /** Extra `«Name»` lines under the stereotype (applied `MetadataDef`s). */
    banners?: string[];
    /** The `size` the server sent (text-metrics size, or a pin's `w`/`h`),
     * copied by `prepareForLayout` before sprotty's hidden render runs. It is
     * authoritative: the measuring pass leaves the element at this size and
     * ELK lays it out with it (`layout.ts`). Absent when the server sent no
     * size — then the DOM measurement is used, as before `REQ-TRS-VIS-017`. */
    serverSize?: ServerSize;
    children?: SysmlNodeChildSchema[];
}

/** An IR port, nested inside its block. */
export interface SysmlPortSchema extends SPort, SysmlShapeFields {
    type: 'port';
    style?: PortStyle;
    /** See `SysmlNodeSchema.serverSize`. */
    serverSize?: ServerSize;
    children?: SysmlNodeChildSchema[];
}

/** What a label child stands for — set by `prepareForLayout` so the label
 * view picks the font, and the ELK configurator its placement. */
export type LabelRole = 'name' | 'stereotype' | 'banner' | 'line' | 'free' | 'edge' | 'keyword';

/** A text label: the synthetic `<id>-label` child of every node/port, or a
 * free IR `kind: label` shape (which then also carries the shape fields). */
export interface SysmlLabelSchema extends SLabel {
    /** `label:edge` for an edge's keyword/label children: same basic type for
     * sprotty-elk, but a model class without sprotty's `edgeLayoutFeature`
     * so the ELK-placed position is honoured (see `container.ts`). */
    type: 'label' | 'label:edge';
    text: string;
    kind?: string;
    ref?: string;
    role?: LabelRole;
    /** See `SysmlNodeSchema.serverSize`. */
    serverSize?: ServerSize;
}

/** An IR compartment: one entry of `lines` per rendered line. */
export interface SysmlCompartmentSchema extends SCompartment {
    type: 'compartment';
    lines: string[];
    kind: string;
    ref: string;
    name: string;
    /** See `SysmlNodeSchema.serverSize`. */
    serverSize?: ServerSize;
    children?: SysmlNodeChildSchema[];
}

/** Anything that can sit inside a node's `children`. */
export type SysmlNodeChildSchema = SysmlNodeSchema | SysmlPortSchema | SysmlLabelSchema | SysmlCompartmentSchema;

/** An IR edge — always a root child. */
export interface SysmlEdgeSchema extends SEdge {
    type: 'edge';
    /** IR edge kind (`EdgeKind::as_str`): `flow`, `binding`, `composition`, `inheritance`, … */
    kind: string;
    ref?: string;
    label?: string;
    style?: EdgeStyle;
    /** `keyword`/`label` as label children (`prepareEdge`) so ELK places them. */
    children?: SysmlLabelSchema[];
}

/** A root child: a top-level IR node (of any sprotty type) or an edge. */
export type SysmlChildSchema = SysmlNodeChildSchema | SysmlEdgeSchema;

/** A connectable shape: the two sprotty types an edge may join. */
export type SysmlShapeSchema = SysmlNodeSchema | SysmlPortSchema;

/** Root JSON returned by `GET /api/diagrams/model/{*qname}`. (`SGraph`'s own
 * `layoutOptions` is sprotty's micro-layout map; ours carries ELK ids and one
 * string list, hence the `Omit`.) */
export interface DiagramModelSchema extends Omit<SGraph, 'layoutOptions'> {
    type: 'graph';
    qualifiedName: string;
    diagramKind: string;
    subject?: string;
    /** ELK option ids from the IR's layout hints (`elk.algorithm`, `elk.direction`,
     * `elk.hierarchyHandling`, `elk.portConstraints`) plus
     * `syscribe.reversedEdgeKinds`. Consumed by ELK from Phase 2; Phase 0 reads
     * only `elk.direction` for its placement shim. */
    layoutOptions: Record<string, string | string[]>;
    /** Ids of the nodes a human pinned (`layout:` entries). */
    pinned: string[];
    /** Whether the node set was regenerated from `subject` (`vis::derive`)
     * rather than read from a manifest — a derived diagram has no manifest to
     * sync an edge into (`connect-rules.ts`). Absent from an older server,
     * which falls back to the shape-id heuristic. */
    derived?: boolean;
    children: SysmlChildSchema[];
}

export function isNodeSchema(child: SysmlChildSchema): child is SysmlNodeSchema {
    return child.type === 'node';
}

export function isPortSchema(child: SysmlChildSchema): child is SysmlPortSchema {
    return child.type === 'port';
}

/** `node` or `port` — the shapes an edge can connect and a drag can move. */
export function isShapeSchema(child: SysmlChildSchema): child is SysmlShapeSchema {
    return child.type === 'node' || child.type === 'port';
}

export function isEdgeSchema(child: SysmlChildSchema): child is SysmlEdgeSchema {
    return child.type === 'edge';
}

export function isLabelSchema(child: SysmlChildSchema): child is SysmlLabelSchema {
    return child.type === 'label' || child.type === 'label:edge';
}

export function isCompartmentSchema(child: SysmlChildSchema): child is SysmlCompartmentSchema {
    return child.type === 'compartment';
}

// ---------------------------------------------------------------------------
// Tree helpers — the schema is nested, so every lookup walks the tree.
// ---------------------------------------------------------------------------

/** The children list of a root or node, or an empty list. */
export function childrenOf(parent: DiagramModelSchema | SysmlNodeChildSchema): SysmlChildSchema[] {
    return ((parent as { children?: SysmlChildSchema[] }).children ?? []) as SysmlChildSchema[];
}

/** Depth-first walk over every element below `parent` (not `parent` itself),
 * yielding each with its immediate container. */
export function* walkTree(
    parent: DiagramModelSchema | SysmlNodeChildSchema,
): Generator<{ element: SysmlChildSchema; container: DiagramModelSchema | SysmlNodeChildSchema }> {
    for (const element of childrenOf(parent)) {
        yield { element, container: parent };
        if (!isEdgeSchema(element)) {
            yield* walkTree(element);
        }
    }
}

/** Find a shape (`node`/`port`) anywhere in the tree by id. */
export function findShape(model: DiagramModelSchema, id: string): SysmlShapeSchema | undefined {
    for (const { element } of walkTree(model)) {
        if (element.id === id && isShapeSchema(element)) {
            return element;
        }
    }
    return undefined;
}

/** The immediate container of the element with `id` (the root or a node), if any. */
export function containerOf(
    model: DiagramModelSchema,
    id: string,
): DiagramModelSchema | SysmlNodeChildSchema | undefined {
    for (const { element, container } of walkTree(model)) {
        if (element.id === id) {
            return container;
        }
    }
    return undefined;
}

/** Every shape (`node`/`port`) in the tree, in document order. */
export function allShapes(model: DiagramModelSchema): SysmlShapeSchema[] {
    const out: SysmlShapeSchema[] = [];
    for (const { element } of walkTree(model)) {
        if (isShapeSchema(element)) {
            out.push(element);
        }
    }
    return out;
}

/** Ids of `element` and everything below it. */
export function subtreeIds(element: SysmlChildSchema): string[] {
    const ids = [element.id];
    if (!isEdgeSchema(element)) {
        for (const { element: e } of walkTree(element)) {
            ids.push(e.id);
        }
    }
    return ids;
}

/** Remove every element whose id is in `ids` from wherever it sits in the tree. */
export function removeFromTree(model: DiagramModelSchema, ids: string[]): void {
    const prune = (parent: DiagramModelSchema | SysmlNodeChildSchema): void => {
        const kids = (parent as { children?: SysmlChildSchema[] }).children;
        if (!kids) {
            return;
        }
        (parent as { children: SysmlChildSchema[] }).children = kids.filter(c => !ids.includes(c.id));
        for (const c of (parent as { children: SysmlChildSchema[] }).children) {
            if (!isEdgeSchema(c)) {
                prune(c);
            }
        }
    };
    prune(model);
}

// ---------------------------------------------------------------------------
// Guarded-write response shape (`routes::mutate::WriteResponse`,
// `crates/syscribe-server/src/routes/mutate.rs`) — every mutating endpoint
// (create/delete element, add/remove connection) returns this.
// ---------------------------------------------------------------------------

export interface Finding {
    code: string;
    severity: 'error' | 'warning';
    file: string;
    message: string;
}

export interface BlockedByEntry {
    qname: string;
    id?: string | null;
}

export interface WriteResponse {
    written: boolean;
    newErrors: Finding[];
    resolvedErrors: Finding[];
    newWarnings: Finding[];
    resolvedWarnings: Finding[];
    diff: string;
    reason?: string | null;
    /** Populated only by `delete_element`'s referrer-blocked refusal; empty
     * (but always present) for every other outcome — see
     * `routes::mutate::WriteResponse`'s doc comment for the one
     * always-200/`written:false` convention every guarded write follows now. */
    blockedBy?: BlockedByEntry[];
}
