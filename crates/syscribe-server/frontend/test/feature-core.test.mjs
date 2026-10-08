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

const conf = (features, extra = {}) => ({
    hasFeatureModel: true, satisfiable: true, features: Object.fromEntries(Object.entries(features).map(([k, v]) => [k, { state: v }])), conflict: null,
    products: { count: 6, capped: false }, completion: { 'F::Car': true, 'F::Engine': true, 'F::Petrol': false }, unknown: [], skipped: null, featureModel: 'F', ...extra,
});

scenario('a click cycles undecided, selected, deselected, undecided', () => {
    assert.equal(core.nextChoice(undefined), true);
    assert.equal(core.nextChoice(true), false);
    assert.equal(core.nextChoice(false), undefined);
});

scenario('withChoice sets and removes a choice without modifying the original', () => {
    const a = { x: true };
    const b = core.withChoice(a, 'y', false);
    assert.deepEqual(b, { x: true, y: false });
    assert.deepEqual(core.withChoice(b, 'x', undefined), { y: false });
    assert.deepEqual(a, { x: true });
});

scenario('the configurator state is applied to the diagram by qualified name and cleared without a result', () => {
    const copy = JSON.parse(JSON.stringify(model));
    core.applyConfiguration(copy, conf({ 'F::Car': 'forcedOn', 'F::Petrol': 'selected', 'F::Electric': 'forcedOff' }));
    const state = id => copy.children.find(c => c.id === id).config;
    assert.equal(state('car'), 'forcedOn');
    assert.equal(state('petrol'), 'selected');
    assert.equal(state('electric'), 'forcedOff');
    assert.equal(state('engine'), undefined);
    core.applyConfiguration(copy, null);
    assert.equal(state('car'), undefined);
});

scenario('the product count reads naturally', () => {
    assert.equal(core.productsText(conf({})), '6 valid products');
    assert.equal(core.productsText(conf({}, { products: { count: 1, capped: false } })), '1 valid product');
    assert.equal(core.productsText(conf({}, { products: { count: 0, capped: false } })), 'no valid product');
    assert.equal(core.productsText(conf({}, { products: { count: 10000, capped: true } })), 'at least 10,000 valid products');
    assert.equal(core.productsText(null), '');
});

scenario('a conflict names the clashing choices and the constraints, in plain words', () => {
    const r = conf({}, { satisfiable: false, conflict: { choices: [{ feature: 'F::Charger', selected: false }, { feature: 'F::Electric', selected: true }], constraints: ["'F::Electric' requires 'F::Charger'"] } });
    const text = core.describeConflict(r, q => q.split('::').pop());
    assert.equal(text, "No valid product has deselecting Charger and selecting Electric ('F::Electric' requires 'F::Charger').");
    const one = conf({}, { satisfiable: false, conflict: { choices: [{ feature: 'F::Petrol', selected: true }], constraints: [] } });
    assert.equal(core.describeConflict(one, q => q.split('::').pop()), 'No valid product has selecting Petrol.');
});

scenario('the counts split chosen, implied and open features', () => {
    const r = conf({ a: 'selected', b: 'deselected', c: 'forcedOn', d: 'forcedOff', e: 'free', f: 'free' });
    assert.deepEqual(core.configCounts(r), { chosen: 2, implied: 2, open: 2 });
    assert.deepEqual(core.configCounts(null), { chosen: 0, implied: 0, open: 0 });
});

scenario('a saved configuration is the complete product the server found, with sorted features and the model package', () => {
    const f = core.configurationFields(conf({}), 'Fast');
    assert.equal(f.name, 'Fast');
    assert.equal(f.status, 'draft');
    assert.equal(f.featureModel, 'F');
    assert.deepEqual(Object.keys(f.features), ['F::Car', 'F::Engine', 'F::Petrol']);
    assert.equal(f.features['F::Petrol'], false);
});

scenario('a new configuration goes beside the stored ones, else at the model root', () => {
    assert.equal(core.configurationPackage([{ qname: 'Configurations::Fast', id: null, name: 'Fast', status: null, selection: {} }]), 'Configurations');
    assert.equal(core.configurationPackage([{ qname: 'Fast', id: null, name: 'Fast', status: null, selection: {} }]), '');
    assert.equal(core.configurationPackage([]), '');
});

scenario('shape ids follow the rule the server uses for derived shapes', () => {
    assert.equal(core.shapeId('Features::Car::Engine'), 's-features-car-engine');
    assert.equal(core.shapeId('UAV::Power::PowerSystem::pdu'), 's-uav-power-powersystem-pdu');
    assert.equal(core.shapeId('A::b_c'), 's-a-b-c');
    assert.equal(core.shapeId('F::Car'), 's-f-car');
});

scenario('the history undoes and redoes in order and a fresh edit forgets the redo', () => {
    const h = new core.EditHistory();
    assert.equal(h.canUndo, false);
    h.record({ op: 'undo-1' });
    h.record({ op: 'undo-2' });
    assert.equal(h.nextUndo().op, 'undo-2');
    h.undone({ op: 'redo-2' });
    assert.equal(h.canRedo, true);
    assert.equal(h.nextUndo().op, 'undo-1');
    assert.equal(h.nextRedo().op, 'redo-2');
    h.redone({ op: 'undo-2b' });
    assert.equal(h.nextUndo().op, 'undo-2b');
    assert.equal(h.canRedo, false);
    h.undone({ op: 'redo-x' });
    h.record({ op: 'undo-3' });
    assert.equal(h.canRedo, false, 'a new edit clears the redo stack');
    h.clear();
    assert.equal(h.canUndo, false);
});

const delta = (o = {}) => ({ worsens: false, becameVoid: false, healedVoid: false, newDead: [], resolvedDead: [], newFalseOptional: [], resolvedFalseOptional: [], newInvalidConfigurations: [], resolvedInvalidConfigurations: [], conflicts: [], countsBefore: {}, countsAfter: {}, ...o });
const short = q => q.split('::').pop();

scenario('a delta is described in plain sentences, worse things only on request', () => {
    const d = delta({ worsens: true, newDead: ['F::Electric'], newFalseOptional: ['F::A', 'F::B'], newInvalidConfigurations: ['CONF-1'], resolvedDead: ['F::Old'] });
    const all = core.deltaLines(d, short);
    assert.deepEqual(all, [
        'Electric becomes dead (in no product).',
        'A, B become false-optional (forced on although optional).',
        'CONF-1 is no longer a valid product.',
        'Old is no longer dead.',
    ]);
    assert.equal(core.deltaLines(d, short, true).length, 3, 'worse only leaves out what improved');
    assert.match(core.deltaLines(delta({ becameVoid: true, worsens: true, conflicts: ["'A' excludes 'B'"] }), short)[0], /becomes void.*'A' excludes 'B'/);
    assert.deepEqual(core.deltaLines(delta({ healedVoid: true }), short), ['The feature model is no longer void.']);
    assert.deepEqual(core.deltaLines(delta(), short), []);
});

scenario('a drop lands on the smallest feature box under the point, never the dragged feature or its descendants', () => {
    const boxes = new Map([['a', { x: 0, y: 0, w: 100, h: 100 }], ['b', { x: 10, y: 10, w: 20, h: 20 }], ['c', { x: 200, y: 0, w: 50, h: 50 }]]);
    assert.equal(core.dropTarget(boxes, 'z', { x: 15, y: 15 }, new Set()), 'b', 'smallest wins');
    assert.equal(core.dropTarget(boxes, 'b', { x: 15, y: 15 }, new Set()), 'a', 'not itself');
    assert.equal(core.dropTarget(boxes, 'z', { x: 15, y: 15 }, new Set(['b'])), 'a', 'not a descendant');
    assert.equal(core.dropTarget(boxes, 'z', { x: 500, y: 500 }, new Set()), null);
});

scenario('descendants are found through the tree edges at any depth', () => {
    assert.deepEqual([...core.descendantIds(model, 'car')].sort(), ['charger', 'electric', 'engine', 'petrol']);
    assert.deepEqual([...core.descendantIds(model, 'engine')].sort(), ['electric', 'petrol']);
    assert.deepEqual([...core.descendantIds(model, 'petrol')], []);
});

scenario('the roots are the features no tree edge points at', () => {
    assert.deepEqual(core.rootIds(model), ['car']);
    const forest = { ...model, children: [...model.children, feat('bike', 'Bike')] };
    assert.deepEqual(core.rootIds(forest), ['car', 'bike']);
});

scenario('expanding a level opens the collapsed features nearest the roots only', () => {
    assert.deepEqual([...core.expandOneLevel(model, new Set(['car', 'engine']))], ['engine']);
    assert.deepEqual([...core.expandOneLevel(model, new Set(['engine']))], []);
    assert.deepEqual([...core.expandOneLevel(model, new Set())], []);
});

scenario('a diagram too wide to read is shown from its first root, a normal one fitted whole', () => {
    const row = n => new Map(Array.from({ length: n }, (_, i) => [`n${i}`, { x: i * 200, y: 0, w: 160, h: 40 }]));
    assert.equal(core.viewMode(row(5), { width: 1100, height: 600 }), 'fit');
    assert.equal(core.viewMode(row(10), { width: 1100, height: 600 }), 'fit', 'ten across fits at about half size');
    assert.equal(core.viewMode(row(30), { width: 1100, height: 600 }), 'roots', 'thirty across would be an unreadable 18%');
    assert.equal(core.viewMode(row(2000), { width: 1100, height: 600 }), 'roots');
    assert.equal(core.viewMode(new Map(), { width: 1100, height: 600 }), 'fit');
    assert.equal(core.viewMode(row(5), { width: 0, height: 0 }), 'fit', 'an unmeasured viewport is not a reason to refuse');
});

const stored = (sel) => ({ selection: sel });

scenario('the matrix has a row per shown feature in tree order, with each configuration\'s choice and the depth', () => {
    const rows = core.configMatrix(model, new Set(), [stored({ 'F::Car': true, 'F::Petrol': true }), stored({ 'F::Car': true, 'F::Petrol': false })]);
    assert.deepEqual(rows.map(r => r.id), ['car', 'engine', 'petrol', 'electric', 'charger']);
    assert.deepEqual(rows.map(r => r.depth), [0, 1, 2, 2, 1]);
    const petrol = rows.find(r => r.id === 'petrol');
    assert.deepEqual(petrol.cells, [true, false]);
    assert.deepEqual(rows.find(r => r.id === 'engine').cells, [undefined, undefined], 'not mentioned');
});

scenario('a collapsed subtree is not in the matrix and a query keeps matches with their ancestors', () => {
    const rows = core.configMatrix(model, new Set(['engine']), [stored({})]);
    assert.deepEqual(rows.map(r => r.id), ['car', 'engine', 'charger']);
    const found = core.configMatrix(model, new Set(), [stored({})], 'petrol');
    assert.deepEqual(found.map(r => r.id), ['car', 'engine', 'petrol']);
    assert.deepEqual(core.configMatrix(model, new Set(), [stored({})], 'nothing here').map(r => r.id), []);
});

scenario('the matrix marks the features two compared configurations disagree on, unmentioned meaning off', () => {
    const rows = core.configMatrix(model, new Set(), [stored({ 'F::Car': true, 'F::Petrol': true }), stored({ 'F::Car': true, 'F::Electric': true })], '', [0, 1]);
    const differing = rows.filter(r => r.differs).map(r => r.id);
    assert.deepEqual(differing, ['petrol', 'electric']);
});

scenario('comparing two configurations lists what only one selects and counts the rest', () => {
    const c = core.compareConfigs(model, { 'F::Car': true, 'F::Petrol': true, 'F::Charger': false }, { 'F::Car': true, 'F::Electric': true, 'F::Charger': true });
    assert.deepEqual(c.onlyA, ['F::Petrol']);
    assert.deepEqual(c.onlyB, ['F::Electric', 'F::Charger']);
    assert.equal(c.both, 1);
    assert.equal(c.neither, 1, 'Engine is in neither');
});

scenario('a cell reads as tick, cross or dot', () => {
    assert.equal(core.cellGlyph(true), '✓');
    assert.equal(core.cellGlyph(false), '✗');
    assert.equal(core.cellGlyph(undefined), '·');
});

scenario('the impact summary says what a feature gates, who selects it and what depends on it', () => {
    const i = {
        found: true,
        gates: { direct: 3, byType: [{ type: 'PartDef', count: 2, elements: [] }, { type: 'Requirement', count: 1, elements: [] }], inheritedThroughPackages: [{ package: 'P', elements: 4 }] },
        selectedBy: [{ qname: 'C::A', name: 'A' }], deselectedBy: [], requiredBy: [{ qname: 'F::E', name: 'Electric' }], excludedBy: [], descendants: 2,
    };
    assert.deepEqual(core.impactSummary(i), [
        'Gates 3 elements directly (2 PartDef, 1 Requirement) and 4 more through 1 package.',
        'Selected by 1 configuration, deselected by 0.',
        'Required by Electric.',
        '2 features below it.',
    ]);
    assert.equal(core.impactSummary({ found: true, gates: { direct: 0, byType: [], inheritedThroughPackages: [] }, selectedBy: [], deselectedBy: [] })[0], 'Gates no element: nothing has an appliesWhen that names it.');
    assert.deepEqual(core.impactSummary({ found: false }), []);
});

scenario('a parameter form round-trips a declaration and keeps the keys it does not show', () => {
    const decl = { name: 'kw', type: 'ScalarValues::Real', range: '50..=300', default: 120, isRequired: true, enumValues: [1, 2], bindingTime: 'load' };
    const form = core.formOf(decl);
    assert.deepEqual(form, { name: 'kw', type: 'ScalarValues::Real', range: '50..=300', defaultValue: '120', required: true });
    assert.deepEqual(core.buildParameter(decl, form), decl, 'unchanged form, unchanged declaration');
    const changed = core.buildParameter(decl, { ...form, range: '50..=400', defaultValue: '', required: false });
    assert.equal(changed.range, '50..=400');
    assert.equal('default' in changed, false, 'an emptied field removes its key');
    assert.equal('isRequired' in changed, false);
    assert.deepEqual(changed.enumValues, [1, 2], 'unshown keys survive');
    assert.equal(changed.bindingTime, 'load');
});

scenario('a new parameter is built from the form alone, defaults typed as numbers or booleans', () => {
    assert.deepEqual(core.buildParameter(undefined, { name: ' amps ', type: '', range: '', defaultValue: '2.5', required: false }), { name: 'amps', default: 2.5 });
    assert.equal(core.parseDefault('12'), 12);
    assert.equal(core.parseDefault('-3.5'), -3.5);
    assert.equal(core.parseDefault('true'), true);
    assert.equal(core.parseDefault('auto'), 'auto');
    assert.equal(core.formOf(undefined).name, '');
});

scenario('a parameter reads as one line', () => {
    assert.equal(core.parameterSummary({ name: 'kw', type: 'ScalarValues::Real', range: '50..=300', default: 120, isRequired: true }), 'kw: Real [50..=300] = 120, required');
    assert.equal(core.parameterSummary({ name: 'flag' }), 'flag');
});

console.log(`feature-core: ok (${n} scenarios)`);
