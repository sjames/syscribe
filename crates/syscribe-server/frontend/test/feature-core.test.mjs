// `npm test` (REQ-TRS-FMED-001, -002): the pure model logic of the feature
// viewer — collapse and expand, search and reveal, the analysis overlay and
// the banner and summary text (`src/feature-core.ts`), bundled with the local
// esbuild into `test/.build/feature-core.mjs`; no DOM.

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..');
const outfile = path.join(here, '.build', 'feature-core.mjs');
execFileSync(
    path.join(root, 'node_modules', '.bin', 'esbuild'),
    [path.join(root, 'src', 'feature-core.ts'), '--bundle', '--format=esm', '--platform=node', '--log-level=warning', `--outfile=${outfile}`],
    { stdio: 'inherit' },
);
const core = await import(pathToFileURL(outfile).href);

let n = 0;
function scenario(name, fn) {
    fn();
    n += 1;
    console.log(`  ok - ${name}`);
}

const feat = (id, name, extra = {}) => ({ type: 'node', id, kind: 'feature', ref: `F::${name}`, name, feature: { mandatory: false, group: 'optional', childCount: 0, id: `FEAT-${name.toUpperCase()}`, ...extra }, children: [] });
const edge = (kind, a, b) => ({ type: 'edge', id: `e-${kind}-${a}-${b}`, kind, sourceId: a, targetId: b });
// Car > (Engine > (Petrol, Electric), Charger), Electric requires Charger.
const model = {
    id: 'g', type: 'graph', diagramKind: 'FeatureModel', layoutOptions: {}, pinned: [], children: [
        feat('car', 'Car', { childCount: 2 }), feat('engine', 'Engine', { childCount: 2, group: 'alternative' }), feat('petrol', 'Petrol'), feat('electric', 'Electric'), feat('charger', 'Charger'),
        edge('child', 'car', 'engine'), edge('child', 'car', 'charger'), edge('child', 'engine', 'petrol'), edge('child', 'engine', 'electric'),
        edge('requires', 'electric', 'charger'), edge('excludes', 'petrol', 'electric'),
    ],
};
const ids = m => m.children.filter(c => c.type === 'node').map(c => c.id);
const kinds = m => m.children.filter(c => c.type === 'edge').map(c => c.kind);

scenario('the tree is read from the child edges', () => {
    assert.deepEqual([...core.childMap(model).get('engine')], ['petrol', 'electric']);
    assert.equal(core.parentMap(model).get('petrol'), 'engine');
    assert.deepEqual(core.collapsible(model).sort(), ['car', 'engine']);
});

scenario('collapsing a feature hides its subtree and the edges that touch it, and counts what it hid', () => {
    const v = core.visibleModel(model, new Set(['engine']));
    assert.deepEqual(ids(v), ['car', 'engine', 'charger']);
    const engine = v.children.find(c => c.id === 'engine');
    assert.equal(engine.collapsedCount, 2);
    assert.deepEqual(kinds(v).sort(), ['child', 'child'], 'requires and excludes touched hidden features');
    assert.equal(model.children.length, 11, 'the full model is not modified');
});

scenario('collapsing an ancestor wins over a collapsed descendant', () => {
    const v = core.visibleModel(model, new Set(['engine', 'car']));
    assert.deepEqual(ids(v), ['car']);
    assert.equal(v.children.find(c => c.id === 'car').collapsedCount, 4);
});

scenario('an expanded model is unchanged apart from carrying no collapse counts', () => {
    const v = core.visibleModel(model, new Set());
    assert.deepEqual(ids(v), ids(model));
    assert.equal(kinds(v).length, 6);
    assert.ok(v.children.every(c => c.collapsedCount === undefined));
});

scenario('collapseBelow keeps the first levels', () => {
    assert.deepEqual([...core.collapseBelow(model, 1)].sort(), ['car', 'engine']);
    assert.deepEqual([...core.collapseBelow(model, 2)], ['engine']);
    assert.deepEqual([...core.collapseBelow(model, 3)], []);
});

scenario('search matches name, id and qualified name, case-insensitively, in tree order', () => {
    assert.deepEqual(core.search(model, 'ELEC').map(m => m.id), ['electric']);
    assert.deepEqual(core.search(model, 'feat-charger').map(m => m.id), ['charger']);
    assert.deepEqual(core.search(model, 'f::car').map(m => m.id), ['car']);
    assert.deepEqual(core.search(model, '  ').map(m => m.id), [], 'a blank query matches nothing');
    assert.deepEqual(core.search(model, '::e').map(m => m.id), ['engine', 'electric'], 'in the order the diagram lists them');
});

scenario('revealing a match expands exactly its collapsed ancestors', () => {
    const matches = core.search(model, 'petrol');
    const open = core.revealing(model, new Set(['car', 'engine', 'charger']), matches);
    assert.deepEqual([...open].sort(), ['charger']);
});

scenario('the analysis overlay marks each feature by qualified name and clears without a report', () => {
    const copy = JSON.parse(JSON.stringify(model));
    core.applyAnalysis(copy, { hasFeatureModel: true, features: { 'F::Electric': { state: 'dead', reasons: ['x'] }, 'F::Car': { state: 'core', reasons: [] } } });
    const state = id => copy.children.find(c => c.id === id).analysis;
    assert.equal(state('electric'), 'dead');
    assert.equal(state('car'), 'core');
    assert.equal(state('petrol'), undefined);
    core.applyAnalysis(copy, null);
    assert.equal(state('electric'), undefined);
});

scenario('the banner appears for a void model or a skipped analysis and not otherwise', () => {
    const base = { hasFeatureModel: true, void: false, skipped: null, features: {}, conflicts: [], diagnoses: [], invalidConfigurations: [], counts: { features: 5, dead: 0, core: 2, falseOptional: 0 } };
    assert.equal(core.bannerText(null), null);
    assert.equal(core.bannerText(base), null);
    assert.match(core.bannerText({ ...base, void: true, conflicts: ["'A' excludes 'B'"] }), /void.*'A' excludes 'B'/);
    assert.equal(core.bannerText({ ...base, skipped: 'too big' }), 'too big');
    assert.equal(core.bannerText({ ...base, hasFeatureModel: false }), null);
});

scenario('the summary lists the counts and the invalid configurations', () => {
    const a = { hasFeatureModel: true, void: false, skipped: null, features: {}, conflicts: [], diagnoses: [], invalidConfigurations: ['CONF-1', 'CONF-2'], counts: { features: 12, dead: 1, core: 3, falseOptional: 2 } };
    const lines = core.summaryLines(a);
    assert.equal(lines[0], '12 features');
    assert.ok(lines.includes('3 core (in every product)') && lines.includes('1 dead (in no product)') && lines.includes('2 false-optional'));
    assert.ok(lines.at(-1).startsWith('2 invalid configurations: CONF-1, CONF-2'));
    assert.deepEqual(core.summaryLines(null), []);
});

console.log(`feature-core: ok (${n} scenarios)`);
