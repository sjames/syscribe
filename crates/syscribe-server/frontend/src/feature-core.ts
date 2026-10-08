// Pure model logic of the feature model viewer (`REQ-TRS-FMED-001`, `-002`):
// collapse and expand over the feature tree, search, and the analysis overlay.
// No DOM, no sprotty: `feature-main.ts` wires it to the page and
// `test/feature-core.test.mjs` drives it directly.
import {
    DiagramModelSchema,
    FeatureState,
    isEdgeSchema,
    isNodeSchema,
    SysmlChildSchema,
    SysmlEdgeSchema,
    SysmlNodeSchema,
} from './types';

/** The analysis report as `GET /api/feature-model/analysis` returns it. */
export interface FeatureAnalysis {
    hasFeatureModel: boolean;
    void: boolean;
    skipped: string | null;
    features: Record<string, { state: FeatureState; core?: boolean; reasons: string[] }>;
    conflicts: string[];
    diagnoses: string[][];
    invalidConfigurations: string[];
    counts: { features: number; dead: number; core: number; falseOptional: number };
}

/** Feature nodes of a full diagram model, in the order sent. */
export function featureNodes(model: DiagramModelSchema): SysmlNodeSchema[] {
    return model.children.filter(isNodeSchema).filter(n => n.kind === 'feature');
}

/** `parent id → child ids`, from the tree edges (kind `child`). */
export function childMap(model: DiagramModelSchema): Map<string, string[]> {
    const m = new Map<string, string[]>();
    for (const e of model.children.filter(isEdgeSchema)) {
        if (e.kind === 'child') {
            const list = m.get(e.sourceId) ?? [];
            list.push(e.targetId);
            m.set(e.sourceId, list);
        }
    }
    return m;
}

/** `child id → parent id`. */
export function parentMap(model: DiagramModelSchema): Map<string, string> {
    const m = new Map<string, string>();
    for (const e of model.children.filter(isEdgeSchema)) {
        if (e.kind === 'child') {
            m.set(e.targetId, e.sourceId);
        }
    }
    return m;
}

/** Ids of the features with children (the ones that can collapse). */
export function collapsible(model: DiagramModelSchema): string[] {
    return [...childMap(model).keys()];
}

/** The ids to collapse so that only the first `keepDepth` levels show: every
 * feature at depth `keepDepth - 1` that has children. */
export function collapseBelow(model: DiagramModelSchema, keepDepth: number): Set<string> {
    const kids = childMap(model);
    const parents = parentMap(model);
    const depth = (id: string): number => {
        let d = 0;
        for (let p = parents.get(id); p !== undefined; p = parents.get(p)) {
            d += 1;
        }
        return d;
    };
    const out = new Set<string>();
    for (const id of kids.keys()) {
        if (depth(id) >= keepDepth - 1) {
            out.add(id);
        }
    }
    return out;
}

function descendants(id: string, kids: Map<string, string[]>, into: Set<string>): void {
    for (const c of kids.get(id) ?? []) {
        if (!into.has(c)) {
            into.add(c);
            descendants(c, kids, into);
        }
    }
}

/** A copy of `full` with the descendants of every collapsed feature, and the
 * edges touching them, removed. A collapsed feature records how many it hides
 * in `collapsedCount`; a cross-tree edge into a hidden feature is dropped
 * (the Inspector still lists it). The input is not modified. */
export function visibleModel(full: DiagramModelSchema, collapsed: ReadonlySet<string>): DiagramModelSchema {
    const kids = childMap(full);
    const hidden = new Set<string>();
    const counts = new Map<string, number>();
    for (const id of collapsed) {
        if (hidden.has(id)) {
            continue;
        }
        const d = new Set<string>();
        descendants(id, kids, d);
        counts.set(id, d.size);
        for (const x of d) {
            hidden.add(x);
        }
    }
    const children: SysmlChildSchema[] = [];
    for (const c of full.children) {
        if (isNodeSchema(c)) {
            if (hidden.has(c.id)) {
                continue;
            }
            const n = c as SysmlNodeSchema;
            children.push({ ...n, collapsedCount: collapsed.has(n.id) ? (counts.get(n.id) ?? 0) : undefined });
        } else if (isEdgeSchema(c)) {
            const e = c as SysmlEdgeSchema;
            if (hidden.has(e.sourceId) || hidden.has(e.targetId)) {
                continue;
            }
            children.push(e);
        } else {
            children.push(c);
        }
    }
    return { ...full, children };
}

/** Mark every feature with its analysis state (and clear it when there is no report). */
export function applyAnalysis(model: DiagramModelSchema, analysis: FeatureAnalysis | null): void {
    for (const n of featureNodes(model)) {
        const a = analysis?.features[n.ref];
        n.analysis = a ? a.state : undefined;
    }
}

/** The features matching `query` (case-insensitive substring of name, id or
 * qualified name), in tree order. An empty query matches nothing. */
export function search(model: DiagramModelSchema, query: string): SysmlNodeSchema[] {
    const q = query.trim().toLowerCase();
    if (q === '') {
        return [];
    }
    return featureNodes(model).filter(
        n => n.name.toLowerCase().includes(q) || n.ref.toLowerCase().includes(q) || (n.feature?.id ?? '').toLowerCase().includes(q),
    );
}

/** The collapsed set with every ancestor of each match expanded, so the match is visible. */
export function revealing(full: DiagramModelSchema, collapsed: ReadonlySet<string>, matches: readonly SysmlNodeSchema[]): Set<string> {
    const parents = parentMap(full);
    const out = new Set(collapsed);
    for (const m of matches) {
        for (let p = parents.get(m.id); p !== undefined; p = parents.get(p)) {
            out.delete(p);
        }
    }
    return out;
}

/** The text of the banner above the diagram, or `null` when the model is sound. */
export function bannerText(a: FeatureAnalysis | null): string | null {
    if (!a || !a.hasFeatureModel) {
        return null;
    }
    if (a.void) {
        return 'This feature model is void: no valid product exists. ' + (a.conflicts.length ? 'Conflicting constraints: ' + a.conflicts.join('; ') + '.' : '');
    }
    if (a.skipped) {
        return a.skipped;
    }
    return null;
}

/** One line per fact for the summary panel. */
export function summaryLines(a: FeatureAnalysis | null): string[] {
    if (!a || !a.hasFeatureModel) {
        return [];
    }
    const c = a.counts;
    const out = [`${c.features} features`];
    out.push(`${c.core} core (in every product)`);
    out.push(`${c.dead} dead (in no product)`);
    out.push(`${c.falseOptional} false-optional`);
    if (a.invalidConfigurations.length > 0) {
        out.push(`${a.invalidConfigurations.length} invalid configuration${a.invalidConfigurations.length === 1 ? '' : 's'}: ${a.invalidConfigurations.join(', ')}`);
    }
    return out;
}
