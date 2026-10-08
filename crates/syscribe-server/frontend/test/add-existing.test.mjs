// `npm test` (REQ-TRS-VIS-024, TC-TRS-VIS-024): the pure logic of the Add
// existing element picker in `src/add-existing.ts` — which elements a query
// offers, in what order, and the request built from a choice. Bundled with the
// local esbuild into `test/.build/add-existing.mjs` (gitignored); no DOM.

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..');
const outfile = path.join(here, '.build', 'add-existing.mjs');
execFileSync(
    path.join(root, 'node_modules', '.bin', 'esbuild'),
    [path.join(root, 'src', 'add-existing.ts'), '--bundle', '--format=esm', '--platform=node', '--log-level=warning', `--outfile=${outfile}`],
    { stdio: 'inherit' },
);
const { searchElements, buildAddRequest, describe, MAX_RESULTS, DERIVED_MESSAGE } = await import(pathToFileURL(outfile).href);

let scenarios = 0;
function scenario(name, fn) {
    fn();
    scenarios += 1;
    console.log(`  ok - ${name}`);
}

const E = (qualifiedName, elementType, name) => ({ qualifiedName, elementType, name: name ?? qualifiedName.split('::').pop() });
const model = [
    E('UAV', 'Package'),
    E('UAV::Power::PowerSystem', 'PartDef'),
    E('UAV::Power::PowerSystem::pdu', 'Part'),
    E('UAV::Avionics::PowerMonitor', 'PartDef'),
    E('UAV::Power::BatteryPack', 'PartDef'),
    E('Diagrams::PowerIBD', 'Diagram'),
    E('Requirements::PowerReq', 'Requirement', 'Power budget'),
    E('', 'Package'),
];

scenario('an empty query lists elements alphabetically, never diagrams or the root', () => {
    const all = searchElements(model, '');
    assert.deepEqual(all.map(e => e.qualifiedName), [
        'Requirements::PowerReq',
        'UAV',
        'UAV::Avionics::PowerMonitor',
        'UAV::Power::BatteryPack',
        'UAV::Power::PowerSystem',
        'UAV::Power::PowerSystem::pdu',
    ]);
});

scenario('matching is a case-insensitive substring over qualified name and name', () => {
    assert.deepEqual(searchElements(model, 'battery').map(e => e.qualifiedName), ['UAV::Power::BatteryPack']);
    assert.deepEqual(searchElements(model, 'POWER BUDGET').map(e => e.qualifiedName), ['Requirements::PowerReq'], 'by name');
    assert.deepEqual(searchElements(model, 'nothing-like-this'), []);
});

scenario('prefix matches rank before substring matches, exact first', () => {
    const q = searchElements(model, 'power').map(e => e.qualifiedName);
    // Prefix of a name or qualified name first (PowerMonitor, PowerReq's name "Power budget"), then substrings.
    assert.ok(q.indexOf('UAV::Avionics::PowerMonitor') < q.indexOf('UAV::Power::BatteryPack'), q.join(', '));
    assert.equal(searchElements(model, 'UAV')[0].qualifiedName, 'UAV', 'an exact qualified name is first');
});

scenario('results are bounded', () => {
    const many = Array.from({ length: 500 }, (_, i) => E(`P::E${String(i).padStart(3, '0')}`, 'PartDef'));
    assert.equal(searchElements(many, '').length, MAX_RESULTS);
    assert.equal(searchElements(many, 'E4').length, 100);
    assert.equal(searchElements(many, '', 7).length, 7);
});

scenario('a chosen element builds an unpinned request, labelled', () => {
    const r = buildAddRequest({ ref: ' UAV::Power::PowerSystem ', all: model });
    assert.equal(r.ok, true);
    assert.deepEqual(r.request, { ref: 'UAV::Power::PowerSystem' }, 'no x/y: the server leaves the shape unpinned');
    assert.equal(r.label, 'PowerSystem');
});

scenario('nothing chosen, an unknown element and a diagram are refused with a reason', () => {
    const none = buildAddRequest({ ref: '  ', all: model });
    assert.equal(none.ok, false);
    assert.match(none.error, /Choose an element/);
    const typo = buildAddRequest({ ref: 'UAV::Power::PowerSistem', all: model });
    assert.equal(typo.ok, false);
    assert.match(typo.error, /not an element of the model/);
    const diagram = buildAddRequest({ ref: 'Diagrams::PowerIBD', all: model });
    assert.equal(diagram.ok, false);
    assert.match(diagram.error, /diagram cannot be a shape of a diagram/);
});

scenario('the picker shows the type and a derived diagram gets an explanation', () => {
    assert.equal(describe(E('A::B', 'PartDef')), 'PartDef');
    assert.equal(describe({ qualifiedName: 'A::B' }), 'element');
    assert.match(DERIVED_MESSAGE, /derived from its subject/);
    assert.match(DERIVED_MESSAGE, /include:\/exclude:/);
});

console.log(`add-existing: ok (${scenarios} scenarios)`);
