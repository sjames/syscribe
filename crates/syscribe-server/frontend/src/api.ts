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

/** One pin: a parent-relative position, optionally with the size ELK gave
 * the shape (*Pin all* sends both; a drag sends only `x`/`y`). `null`
 * removes the pin (`REQ-TRS-VIS-006`). */
export type LayoutPin = { x: number; y: number; w?: number; h?: number } | null;

/** `PATCH /api/diagrams/layout/{qname}` — every entry written is a pin; the
 * response is a `WriteResponse` (the guarded-write engine). */
export async function patchLayout(diagramQname: string, pins: Record<string, LayoutPin>): Promise<WriteResponse> {
    const resp = await fetch('/api/diagrams/layout/' + qnameToPath(diagramQname), {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(pins),
    });
    if (!resp.ok) {
        throw new Error(`PATCH layout failed (${resp.status})`);
    }
    return asJson<WriteResponse>(resp);
}

/** `DELETE /api/diagrams/layout/{qname}` — removes every pin of the diagram
 * (*Auto-layout*, `REQ-TRS-VIS-007`/`011`). */
export async function deleteLayout(diagramQname: string): Promise<WriteResponse> {
    const resp = await fetch('/api/diagrams/layout/' + qnameToPath(diagramQname), { method: 'DELETE' });
    if (!resp.ok) {
        throw new Error(`DELETE layout failed (${resp.status})`);
    }
    return asJson<WriteResponse>(resp);
}

/** `PUT /api/diagrams/svg/{qname}` — writes the serialised render to the
 * diagram's companion `svgFile:` (*Save companion SVG*, `REQ-TRS-VIS-011`). */
export async function putSvg(diagramQname: string, svg: string): Promise<WriteResponse> {
    const resp = await fetch('/api/diagrams/svg/' + qnameToPath(diagramQname), {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ svg }),
    });
    if (!resp.ok) {
        throw new Error(`PUT svg failed (${resp.status})`);
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
