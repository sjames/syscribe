// Pure model logic of the feature model viewer (`REQ-TRS-FMED-001`, `-002`):
// collapse and expand over the feature tree, search, and the analysis overlay.
// No DOM, no sprotty: `feature-main.ts` wires it to the page and
// `test/feature-core.test.mjs` drives it directly.
import {
    ConfigState,
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

// ---------------------------------------------------------------------------
// The configurator (`REQ-TRS-FMED-003`)
// ---------------------------------------------------------------------------

/** What `POST /api/feature-model/configure` returns. */
export interface ConfigureResult {
    hasFeatureModel: boolean;
    satisfiable: boolean;
    features: Record<string, { state: ConfigState }>;
    conflict: { choices: { feature: string; selected: boolean }[]; constraints: string[] } | null;
    products: { count: number; capped: boolean } | null;
    completion: Record<string, boolean>;
    unknown: string[];
    skipped: string | null;
    featureModel: string;
}

/** A stored `Configuration` as `GET /api/feature-model/configurations` lists it. */
export interface StoredConfiguration {
    id: string | null;
    qname: string;
    name: string;
    status: string | null;
    selection: Record<string, boolean>;
}

/** The choice a click on a feature leads to: undecided, then selected, then
 * deselected, then undecided again. */
export function nextChoice(current: boolean | undefined): boolean | undefined {
    if (current === undefined) {
        return true;
    }
    return current ? false : undefined;
}

/** `choices` with `feature` set to `value` (removed when `undefined`); the input is not modified. */
export function withChoice(choices: Readonly<Record<string, boolean>>, feature: string, value: boolean | undefined): Record<string, boolean> {
    const out = { ...choices };
    if (value === undefined) {
        delete out[feature];
    } else {
        out[feature] = value;
    }
    return out;
}

/** Mark every feature with its configurator state, or clear it without a result. */
export function applyConfiguration(model: DiagramModelSchema, result: ConfigureResult | null): void {
    for (const n of featureNodes(model)) {
        n.config = result?.features[n.ref]?.state;
    }
}

/** The count line: `6 valid products`, `at least 10,000 valid products`, `1 valid product`, `no valid product`. */
export function productsText(r: ConfigureResult | null): string {
    if (!r || !r.products) {
        return '';
    }
    const { count, capped } = r.products;
    const n = count.toLocaleString('en-US');
    if (capped) {
        return `at least ${n} valid products`;
    }
    return count === 0 ? 'no valid product' : `${n} valid product${count === 1 ? '' : 's'}`;
}

/** The sentence that explains a refused choice: which choices clash and the constraints they clash with. */
export function describeConflict(r: ConfigureResult, nameOf: (qname: string) => string): string {
    const c = r.conflict;
    if (!c) {
        return '';
    }
    const choices = c.choices.map(x => `${x.selected ? 'selecting' : 'deselecting'} ${nameOf(x.feature)}`);
    const joined = choices.length <= 1 ? choices.join('') : choices.slice(0, -1).join(', ') + ' and ' + choices[choices.length - 1];
    const why = c.constraints.length > 0 ? ` (${c.constraints.join('; ')})` : '';
    return `No valid product has ${joined}${why}.`;
}

/** Counts for the status line: how many features are chosen, implied and open. */
export function configCounts(r: ConfigureResult | null): { chosen: number; implied: number; open: number } {
    const out = { chosen: 0, implied: 0, open: 0 };
    for (const f of Object.values(r?.features ?? {})) {
        if (f.state === 'selected' || f.state === 'deselected') {
            out.chosen += 1;
        } else if (f.state === 'forcedOn' || f.state === 'forcedOff') {
            out.implied += 1;
        } else {
            out.open += 1;
        }
    }
    return out;
}

/** The `fields` of the `Configuration` to create from a satisfiable result: the
 * complete product the server found for the choices, written as `features:`. */
export function configurationFields(r: ConfigureResult, name: string): Record<string, unknown> {
    const features: Record<string, boolean> = {};
    for (const q of Object.keys(r.completion).sort()) {
        features[q] = r.completion[q];
    }
    const fields: Record<string, unknown> = { name, status: 'draft', features };
    if (r.featureModel) {
        fields.featureModel = r.featureModel;
    }
    return fields;
}

/** Where a new configuration goes: the package of the stored ones, else the model root. */
export function configurationPackage(stored: readonly StoredConfiguration[]): string {
    const first = stored[0]?.qname;
    if (!first) {
        return '';
    }
    const i = first.lastIndexOf('::');
    return i < 0 ? '' : first.slice(0, i);
}

// ---------------------------------------------------------------------------
// Editing (`REQ-TRS-FMED-004`)
// ---------------------------------------------------------------------------

/** One semantic edit, as `POST /api/feature-model/edit` takes it (`feature_edit::EditOp`). */
export type EditOp = { op: string } & Record<string, unknown>;

/** What the server returns for an edit. */
export interface EditResult {
    written: boolean;
    preview: boolean;
    needsConfirmation: boolean;
    reason: string | null;
    delta: EditDelta | null;
    undo: EditOp | null;
    feature: string | null;
}

export interface EditDelta {
    worsens: boolean;
    becameVoid: boolean;
    healedVoid: boolean;
    newDead: string[];
    resolvedDead: string[];
    newFalseOptional: string[];
    resolvedFalseOptional: string[];
    newInvalidConfigurations: string[];
    resolvedInvalidConfigurations: string[];
    conflicts: string[];
    countsBefore: { features: number; dead: number; core: number; falseOptional: number };
    countsAfter: { features: number; dead: number; core: number; falseOptional: number };
}

/** The diagram id of a feature: `s-` and its qualified name lower-cased with every
 * run of other characters as one `-` (`vis::ir::derived_shape_id`). */
export function shapeId(qname: string): string {
    let out = 's-';
    let pending = false;
    for (const c of qname) {
        if (/[A-Za-z0-9]/.test(c)) {
            if (pending && out.length > 2) {
                out += '-';
            }
            pending = false;
            out += c.toLowerCase();
        } else {
            pending = true;
        }
    }
    return out;
}

/** The undo and redo stacks of a session's edits. Each committed edit hands back
 * the operation that reverses it; undoing runs that operation, which hands back
 * its own reversal, the redo. A new edit clears what could have been redone. */
export class EditHistory {
    private undoable: EditOp[] = [];
    private redoable: EditOp[] = [];

    get canUndo(): boolean {
        return this.undoable.length > 0;
    }
    get canRedo(): boolean {
        return this.redoable.length > 0;
    }

    /** A fresh edit was committed; `undo` reverses it. */
    record(undo: EditOp): void {
        this.undoable.push(undo);
        this.redoable = [];
    }
    /** The operation to run to undo the last edit (it stays on the stack until `undone`). */
    nextUndo(): EditOp | undefined {
        return this.undoable[this.undoable.length - 1];
    }
    /** The undo ran; `redo` reverses it. */
    undone(redo: EditOp): void {
        this.undoable.pop();
        this.redoable.push(redo);
    }
    nextRedo(): EditOp | undefined {
        return this.redoable[this.redoable.length - 1];
    }
    /** The redo ran; `undo` reverses it again. */
    redone(undo: EditOp): void {
        this.redoable.pop();
        this.undoable.push(undo);
    }
    clear(): void {
        this.undoable = [];
        this.redoable = [];
    }
}

/** What an edit does to the model's validity, one sentence per fact, for the
 * confirmation and the toast. `worse` lists only what got worse. */
export function deltaLines(d: EditDelta, nameOf: (q: string) => string, worseOnly = false): string[] {
    const names = (qs: string[]): string => qs.map(nameOf).join(', ');
    const out: string[] = [];
    if (d.becameVoid) {
        out.push('The feature model becomes void: no valid product exists.' + (d.conflicts.length ? ' ' + d.conflicts.join('; ') + '.' : ''));
    }
    if (d.newDead.length) {
        out.push(`${names(d.newDead)} ${d.newDead.length === 1 ? 'becomes' : 'become'} dead (in no product).`);
    }
    if (d.newFalseOptional.length) {
        out.push(`${names(d.newFalseOptional)} ${d.newFalseOptional.length === 1 ? 'becomes' : 'become'} false-optional (forced on although optional).`);
    }
    if (d.newInvalidConfigurations.length) {
        out.push(`${d.newInvalidConfigurations.join(', ')} ${d.newInvalidConfigurations.length === 1 ? 'is' : 'are'} no longer a valid product.`);
    }
    if (!worseOnly) {
        if (d.healedVoid) {
            out.push('The feature model is no longer void.');
        }
        if (d.resolvedDead.length) {
            out.push(`${names(d.resolvedDead)} ${d.resolvedDead.length === 1 ? 'is' : 'are'} no longer dead.`);
        }
        if (d.resolvedFalseOptional.length) {
            out.push(`${names(d.resolvedFalseOptional)} ${d.resolvedFalseOptional.length === 1 ? 'is' : 'are'} no longer false-optional.`);
        }
        if (d.resolvedInvalidConfigurations.length) {
            out.push(`${d.resolvedInvalidConfigurations.join(', ')} ${d.resolvedInvalidConfigurations.length === 1 ? 'is' : 'are'} valid again.`);
        }
    }
    return out;
}

/** The feature a drop lands on: the node whose box holds the drop point, other
 * than the dragged feature and anything below it, preferring the smallest box. */
export function dropTarget(
    boxes: ReadonlyMap<string, { x: number; y: number; w: number; h: number }>,
    dragged: string,
    point: { x: number; y: number },
    descendantsOf: ReadonlySet<string>,
): string | null {
    let best: { id: string; area: number } | null = null;
    for (const [id, b] of boxes) {
        if (id === dragged || descendantsOf.has(id)) {
            continue;
        }
        if (point.x >= b.x && point.x <= b.x + b.w && point.y >= b.y && point.y <= b.y + b.h) {
            const area = b.w * b.h;
            if (!best || area < best.area) {
                best = { id, area };
            }
        }
    }
    return best?.id ?? null;
}

/** Ids of a feature's descendants, from the tree edges. */
export function descendantIds(model: DiagramModelSchema, id: string): Set<string> {
    const kids = childMap(model);
    const out = new Set<string>();
    const walk = (x: string): void => {
        for (const c of kids.get(x) ?? []) {
            if (!out.has(c)) {
                out.add(c);
                walk(c);
            }
        }
    };
    walk(id);
    return out;
}

// ---------------------------------------------------------------------------
// Scale (`REQ-TRS-FMED-007`)
// ---------------------------------------------------------------------------

/** Ids of the features that have no parent: the roots of the forest. */
export function rootIds(model: DiagramModelSchema): string[] {
    const parents = parentMap(model);
    return featureNodes(model).filter(n => !parents.has(n.id)).map(n => n.id);
}

/** The collapsed set with one more level opened: the collapsed features nearest
 * the roots are expanded, so a huge model is read a level at a time. */
export function expandOneLevel(model: DiagramModelSchema, collapsed: ReadonlySet<string>): Set<string> {
    const parents = parentMap(model);
    const depth = (id: string): number => {
        let d = 0;
        for (let p = parents.get(id); p !== undefined; p = parents.get(p)) {
            d += 1;
        }
        return d;
    };
    const out = new Set(collapsed);
    if (collapsed.size === 0) {
        return out;
    }
    const shallowest = Math.min(...[...collapsed].map(depth));
    for (const id of collapsed) {
        if (depth(id) === shallowest) {
            out.delete(id);
        }
    }
    return out;
}

/** The smallest scale at which a feature name can still be read. */
export const MIN_READABLE_SCALE = 0.3;

/** How to show a laid-out diagram: `fit` it whole when that is still readable,
 * else `roots`, zoomed in on the first root, because a model of thousands of
 * features is read by zooming and searching, not seen at once at 1%. */
export function viewMode(
    boxes: ReadonlyMap<string, { x: number; y: number; w: number; h: number }>,
    viewport: { width: number; height: number },
    padding = 30,
): 'fit' | 'roots' {
    if (boxes.size === 0 || viewport.width <= 0 || viewport.height <= 0) {
        return 'fit';
    }
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    for (const b of boxes.values()) {
        minX = Math.min(minX, b.x);
        minY = Math.min(minY, b.y);
        maxX = Math.max(maxX, b.x + b.w);
        maxY = Math.max(maxY, b.y + b.h);
    }
    const scale = Math.min(viewport.width / (maxX - minX + 2 * padding), viewport.height / (maxY - minY + 2 * padding));
    return scale < MIN_READABLE_SCALE ? 'roots' : 'fit';
}

// ---------------------------------------------------------------------------
// Impact, comparison and the matrix (`REQ-TRS-FMED-005`, `-006`)
// ---------------------------------------------------------------------------

/** `GET /api/feature-model/impact`. */
export interface ImpactResult {
    found: boolean;
    feature?: { qname: string; id?: string | null; name: string };
    gates?: {
        direct: number;
        byType: { type: string; count: number; elements: { qname: string; id?: string | null; name: string }[] }[];
        inheritedThroughPackages: { package: string; elements: number }[];
    };
    selectedBy?: { qname: string; id?: string | null; name: string }[];
    deselectedBy?: { qname: string; id?: string | null; name: string }[];
    requires?: { qname: string; name: string }[];
    requiredBy?: { qname: string; name: string }[];
    excludes?: { qname: string; name: string }[];
    excludedBy?: { qname: string; name: string }[];
    children?: number;
    descendants?: number;
}

/** One-line facts for the Impact section, in the order a reader weighs them. */
export function impactSummary(i: ImpactResult): string[] {
    if (!i.found) {
        return [];
    }
    const out: string[] = [];
    const inherited = (i.gates?.inheritedThroughPackages ?? []).reduce((n, p) => n + p.elements, 0);
    const direct = i.gates?.direct ?? 0;
    if (direct + inherited === 0) {
        out.push('Gates no element: nothing has an appliesWhen that names it.');
    } else {
        const parts = (i.gates?.byType ?? []).map(t => `${t.count} ${t.type}`);
        out.push(`Gates ${direct} element${direct === 1 ? '' : 's'} directly (${parts.join(', ')})` + (inherited ? ` and ${inherited} more through ${i.gates?.inheritedThroughPackages.length} package${i.gates?.inheritedThroughPackages.length === 1 ? '' : 's'}.` : '.'));
    }
    const sel = i.selectedBy?.length ?? 0;
    const desel = i.deselectedBy?.length ?? 0;
    out.push(`Selected by ${sel} configuration${sel === 1 ? '' : 's'}, deselected by ${desel}.`);
    if (i.requiredBy?.length) {
        out.push(`Required by ${i.requiredBy.map(f => f.name).join(', ')}.`);
    }
    if (i.excludedBy?.length) {
        out.push(`Excluded by ${i.excludedBy.map(f => f.name).join(', ')}.`);
    }
    if (i.descendants) {
        out.push(`${i.descendants} feature${i.descendants === 1 ? '' : 's'} below it.`);
    }
    return out;
}

/** One configuration's choice for a feature: chosen on, chosen off, or not mentioned. */
export type Cell = boolean | undefined;

export interface MatrixRow {
    id: string;
    qname: string;
    name: string;
    depth: number;
    cells: Cell[];
    /** Two compared configurations disagree on this feature (unmentioned counts as off). */
    differs: boolean;
}

/** The feature-by-configuration matrix: one row per feature the diagram currently
 * shows (in tree order, so collapsed subtrees stay collapsed), one cell per stored
 * configuration. A non-empty `query` keeps the features that match it and their
 * ancestors. `compare` names two configurations (by index) whose differences are marked. */
export function configMatrix(
    full: DiagramModelSchema,
    collapsed: ReadonlySet<string>,
    configs: readonly { selection: Record<string, boolean> }[],
    query = '',
    compare?: [number, number],
): MatrixRow[] {
    const shown = visibleModel(full, collapsed);
    const parents = parentMap(full);
    const depth = (id: string): number => {
        let d = 0;
        for (let p = parents.get(id); p !== undefined; p = parents.get(p)) {
            d += 1;
        }
        return d;
    };
    let keep: Set<string> | null = null;
    if (query.trim() !== '') {
        keep = new Set();
        for (const m of search(full, query)) {
            keep.add(m.id);
            for (let p = parents.get(m.id); p !== undefined; p = parents.get(p)) {
                keep.add(p);
            }
        }
    }
    const rows: MatrixRow[] = [];
    for (const n of featureNodes(shown)) {
        if (keep && !keep.has(n.id)) {
            continue;
        }
        const cells = configs.map(c => c.selection[n.ref]);
        const differs = compare ? (cells[compare[0]] === true) !== (cells[compare[1]] === true) : false;
        rows.push({ id: n.id, qname: n.ref, name: n.name, depth: depth(n.id), cells, differs });
    }
    return rows;
}

export interface Comparison {
    onlyA: string[];
    onlyB: string[];
    both: number;
    neither: number;
}

/** The features selected in `a` only, in `b` only, in both and in neither, over
 * every feature of the model (a feature a configuration does not mention is off). */
export function compareConfigs(full: DiagramModelSchema, a: Record<string, boolean>, b: Record<string, boolean>): Comparison {
    const out: Comparison = { onlyA: [], onlyB: [], both: 0, neither: 0 };
    for (const n of featureNodes(full)) {
        const x = a[n.ref] === true;
        const y = b[n.ref] === true;
        if (x && y) {
            out.both += 1;
        } else if (x) {
            out.onlyA.push(n.ref);
        } else if (y) {
            out.onlyB.push(n.ref);
        } else {
            out.neither += 1;
        }
    }
    return out;
}

/** The glyph of a matrix cell. */
export function cellGlyph(c: Cell): string {
    return c === true ? '✓' : c === false ? '✗' : '·';
}

// ---------------------------------------------------------------------------
// Parameters (`REQ-TRS-FMED-004`)
// ---------------------------------------------------------------------------

/** What the parameter form holds. Every field shows the current value, so an empty one means "none". */
export interface ParameterForm {
    name: string;
    type: string;
    range: string;
    defaultValue: string;
    required: boolean;
}

/** The form's view of a declaration, to prefill it when a parameter is chosen for editing. */
export function formOf(decl: Record<string, unknown> | undefined): ParameterForm {
    const text = (v: unknown): string => (v === undefined || v === null ? '' : String(v));
    return {
        name: text(decl?.name),
        type: text(decl?.type),
        range: text(decl?.range),
        defaultValue: text(decl?.default),
        required: decl?.isRequired === true,
    };
}

/** A default as typed: a number or a boolean when it reads as one, else the text. */
export function parseDefault(text: string): number | boolean | string {
    const t = text.trim();
    if (/^-?\d+(\.\d+)?$/.test(t)) {
        return Number(t);
    }
    if (t === 'true' || t === 'false') {
        return t === 'true';
    }
    return t;
}

/** The declaration to write: the existing one with the form's fields applied (so
 * keys the form does not show, `enumValues:` or `bindingTime:`, are kept), an
 * emptied field removing its key. */
export function buildParameter(existing: Record<string, unknown> | undefined, form: ParameterForm): Record<string, unknown> {
    const out: Record<string, unknown> = { ...(existing ?? {}), name: form.name.trim() };
    const set = (k: string, v: string): void => {
        if (v.trim() === '') {
            delete out[k];
        } else {
            out[k] = v.trim();
        }
    };
    set('type', form.type);
    set('range', form.range);
    if (form.defaultValue.trim() === '') {
        delete out.default;
    } else {
        out.default = parseDefault(form.defaultValue);
    }
    if (form.required) {
        out.isRequired = true;
    } else {
        delete out.isRequired;
    }
    return out;
}

/** One line for a parameter in the list: `kw: Real [50..=300] = 120, required`. */
export function parameterSummary(decl: Record<string, unknown>): string {
    const short = (t: unknown): string => String(t).split('::').pop() ?? '';
    let s = String(decl.name ?? '?');
    if (decl.type !== undefined) {
        s += `: ${short(decl.type)}`;
    }
    if (decl.range !== undefined) {
        s += ` [${decl.range}]`;
    }
    if (decl.default !== undefined) {
        s += ` = ${decl.default}`;
    }
    if (decl.isRequired === true) {
        s += ', required';
    }
    return s;
}
