// Pure logic of the "Add existing element" picker (`REQ-TRS-VIS-024`): which
// model elements the picker offers for a query, and the request sent to
// `POST /api/diagrams/shapes/{qname}`. No DOM, no fetch — `add-existing-dialog.ts`
// wires it to the page and `test/add-existing.test.mjs` drives it directly.

export interface ElementSummary {
    qualifiedName: string;
    name?: string | null;
    elementType?: string | null;
}

/** Element types the picker never offers: a diagram is not a shape of a diagram. */
const NOT_OFFERED = new Set(['Diagram']);

export const MAX_RESULTS = 100;

/** The elements to list for `query`: a case-insensitive substring match over
 * the qualified name and the name, prefix matches before the rest (exact
 * qualified name first), alphabetical within a rank, at most `limit`. An empty
 * query lists the first `limit` elements alphabetically. */
export function searchElements(all: ElementSummary[], query: string, limit: number = MAX_RESULTS): ElementSummary[] {
    const q = query.trim().toLowerCase();
    const scored: { e: ElementSummary; rank: number }[] = [];
    for (const e of all) {
        if (e.qualifiedName === '' || (e.elementType != null && NOT_OFFERED.has(e.elementType))) {
            continue;
        }
        const qn = e.qualifiedName.toLowerCase();
        const nm = (e.name ?? '').toLowerCase();
        let rank: number;
        if (q === '') {
            rank = 3;
        } else if (qn === q) {
            rank = 0;
        } else if (qn.startsWith(q) || nm.startsWith(q)) {
            rank = 1;
        } else if (qn.includes(q) || nm.includes(q)) {
            rank = 2;
        } else {
            continue;
        }
        scored.push({ e, rank });
    }
    scored.sort((a, b) => a.rank - b.rank || a.e.qualifiedName.localeCompare(b.e.qualifiedName));
    return scored.slice(0, limit).map(s => s.e);
}

/** The text shown beside an option: its type, e.g. `PartDef`. */
export function describe(e: ElementSummary): string {
    return e.elementType ?? 'element';
}

export interface AddExistingForm {
    /** What the user typed or picked. */
    ref: string;
    /** Every element the picker could offer (the full list, not just the visible results). */
    all: ElementSummary[];
}

/** `x`/`y` are omitted: the shape is left unpinned and ELK places it. */
export interface AddShapeRequest {
    ref: string;
}

export type AddResult = { ok: true; request: AddShapeRequest; label: string } | { ok: false; error: string };

/** Validate the choice and build the request body. The ref must be one of the
 * elements the picker offers, by qualified name. */
export function buildAddRequest(form: AddExistingForm): AddResult {
    const ref = form.ref.trim();
    if (ref === '') {
        return { ok: false, error: 'Choose an element to add.' };
    }
    const hit = form.all.find(e => e.qualifiedName === ref);
    if (!hit) {
        const near = searchElements(form.all, ref, 3).map(e => e.qualifiedName);
        return {
            ok: false,
            error: `'${ref}' is not an element of the model.` + (near.length > 0 ? ` Did you mean ${near.join(', ')}?` : ''),
        };
    }
    if (hit.elementType != null && NOT_OFFERED.has(hit.elementType)) {
        return { ok: false, error: `'${ref}' is a diagram; a diagram cannot be a shape of a diagram.` };
    }
    return {
        ok: true,
        request: { ref: hit.qualifiedName },
        label: hit.name && hit.name !== '' ? hit.name : hit.qualifiedName.split('::').pop() ?? hit.qualifiedName,
    };
}

/** The refusal shown instead of the picker on a derived diagram, which follows
 * its subject and has no shape list to add to. */
export const DERIVED_MESSAGE =
    'This diagram is derived from its subject, so its shapes follow the model. Narrow or widen it with include:/exclude: in the file, or start a blank diagram with + Diagram.';
