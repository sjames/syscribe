// Thin fetch wrappers over the diagram-editor REST surface
// (`crates/syscribe-server/src/routes/mutate.rs`, `routes/diagram_model.rs`).
// No caching/state here — that lives in `editor.ts`.

import { DiagramModelSchema, WriteResponse } from './types';

function qnameToPath(qname: string): string {
    return qname.replace(/::/g, '/');
}

async function asJson<T>(resp: Response): Promise<T> {
    return (await resp.json()) as T;
}

export async function fetchDiagramModel(qname: string): Promise<DiagramModelSchema> {
    const resp = await fetch('/api/diagrams/model/' + qnameToPath(qname));
    if (!resp.ok) {
        throw new Error(`GET diagram model failed (${resp.status})`);
    }
    return asJson<DiagramModelSchema>(resp);
}

/** Reuses the existing `PATCH /api/diagrams/layout/{qname}` endpoint — its
 * request body is unchanged, but it now returns a `WriteResponse` (the
 * guarded-write engine) instead of the retired always-`{ok:true}` body. */
export async function patchLayout(
    diagramQname: string,
    moves: Record<string, { x: number; y: number }>,
): Promise<WriteResponse> {
    const resp = await fetch('/api/diagrams/layout/' + qnameToPath(diagramQname), {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(moves),
    });
    if (!resp.ok) {
        throw new Error(`PATCH layout failed (${resp.status})`);
    }
    return asJson<WriteResponse>(resp);
}

export interface ShapeDiagramContext {
    qname: string;
    shapeId: string;
    x: number;
    y: number;
    kind: string;
}

export interface CreateElementRequest {
    qname: string;
    type: string;
    fields?: unknown;
    doc?: string;
    diagram?: ShapeDiagramContext;
}

export async function createElement(req: CreateElementRequest): Promise<WriteResponse> {
    const resp = await fetch('/api/elements', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(req),
    });
    return asJson<WriteResponse>(resp);
}

/** Never passes `force` silently — a blocked delete is surfaced to the caller
 * as `written:false` with `blockedBy` populated, same as every other refusal
 * shape in this module (the server always responds `200 OK` now; there is no
 * status-code branch left to take here). */
export async function deleteElement(qname: string): Promise<WriteResponse> {
    const resp = await fetch('/api/elements/' + qnameToPath(qname), { method: 'DELETE' });
    return asJson<WriteResponse>(resp);
}

export interface EdgeDiagramContext {
    qname: string;
    edgeId: string;
    sourceShapeId?: string;
    targetShapeId?: string;
}

export interface AddConnectionRequest {
    qname: string;
    from: string;
    to: string;
    typedBy?: string;
    diagram?: EdgeDiagramContext;
}

export async function addConnection(req: AddConnectionRequest): Promise<WriteResponse> {
    const resp = await fetch('/api/connections', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(req),
    });
    return asJson<WriteResponse>(resp);
}
