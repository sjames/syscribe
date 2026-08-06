// Wire types shared between `crates/syscribe-server/src/routes/diagram_model.rs`
// (the `GET /api/diagrams/model/{*qname}` JSON endpoint) and
// `crates/syscribe-server/src/routes/mutate.rs` (the guarded-write mutation
// endpoints) on one side, and the sprotty client on the other.

import { SEdge, SGraph, SNode } from 'sprotty-protocol';

/** A `shapes:` entry — one sprotty node per diagram shape. */
export interface SysmlNodeSchema extends SNode {
    type: 'node';
    /** The model qualified name this shape depicts (`shapes.<id>.ref`). */
    ref: string;
    /** SysML element kind, e.g. `PartDef`, `Requirement`, `TestCase`. */
    kind: string;
    name: string;
    isAbstract?: boolean;
}

/** An `edges:` entry — one sprotty edge per diagram connection. */
export interface SysmlEdgeSchema extends SEdge {
    type: 'edge';
    /** Semantic edge kind, e.g. `derivedFrom`, `verifies`, `allocatedTo`. */
    kind: string;
    ref?: string;
}

export type SysmlChildSchema = SysmlNodeSchema | SysmlEdgeSchema;

/** Root JSON returned by `GET /api/diagrams/model/{*qname}`. */
export interface DiagramModelSchema extends SGraph {
    type: 'graph';
    qualifiedName: string;
    diagramKind: string;
    subject?: string;
    children: SysmlChildSchema[];
}

export function isNodeSchema(child: SysmlChildSchema): child is SysmlNodeSchema {
    return child.type === 'node';
}

export function isEdgeSchema(child: SysmlChildSchema): child is SysmlEdgeSchema {
    return child.type === 'edge';
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
