// `npm test` (REQ-TRS-VIS-023, TC-TRS-VIS-023): drive the pure form logic of
// the New diagram dialog in `src/new-diagram.ts` — name validation, the kinds
// and their subject types, the default package, and the `POST /api/elements`
// body built for a derived and a blank diagram. Bundled with the local
// esbuild into `test/.build/new-diagram.mjs` (gitignored); no DOM needed.

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..');
const outfile = path.join(here, '.build', 'new-diagram.mjs');
execFileSync(
    path.join(root, 'node_modules', '.bin', 'esbuild'),
    [path.join(root, 'src', 'new-diagram.ts'), '--bundle', '--format=esm', '--platform=node', '--log-level=warning', `--outfile=${outfile}`],
    { stdio: 'inherit' },
);
const { DIAGRAM_KINDS, kindInfo, validateName, defaultPackage, joinQname, tabId, buildCreateRequest } = await import(
    pathToFileURL(outfile).href
);

let scenarios = 0;
function scenario(name, fn) {
    fn();
    scenarios += 1;
    console.log(`  ok - ${name}`);
}

const ibdCandidates = ['UAV::Power::PowerSystem', 'UAV::Power::PowerSystem::pdu'];
const form = (over) => ({
    name: 'PowerIBD',
    kind: 'IBD',
    startFrom: 'derive',
    subject: 'UAV::Power::PowerSystem',
    pkg: 'Diagrams',
    candidates: ibdCandidates,
    ...over,
});

scenario('every offered kind has a generator-valid subject type list', () => {
    assert.deepEqual(
        DIAGRAM_KINDS.map(k => k.kind),
        ['BDD', 'IBD', 'StateMachine', 'Action', 'Sequence', 'Requirement', 'FeatureModel', 'Allocation', 'FaultTree', 'AttackTree', 'SafetyCase'],
    );
    assert.deepEqual(kindInfo('IBD').subjectTypes, ['PartDef', 'Part', 'ItemDef', 'Item']);
    assert.ok(kindInfo('Requirement').subjectTypes.includes('Package'));
    assert.deepEqual(kindInfo('FeatureModel').subjectTypes, ['Package', 'FeatureDef', 'FeatureModel']);
    assert.equal(kindInfo('Mermaid'), undefined, 'hand-authored kinds are not offered');
    for (const k of DIAGRAM_KINDS) {
        assert.ok(k.subjectTypes.length > 0 && k.hint.length > 0, `${k.kind} has subject types and a hint`);
    }
});

scenario('the safety kinds take their subjects from the safety elements and are derived only', () => {
    assert.deepEqual(kindInfo('FaultTree').subjectTypes, ['FaultTree', 'SafetyGoal']);
    assert.deepEqual(kindInfo('AttackTree').subjectTypes, ['AttackTree', 'ThreatScenario']);
    assert.deepEqual(kindInfo('SafetyCase').subjectTypes, ['SafetyGoal', 'Package']);
    for (const k of ['FaultTree', 'AttackTree', 'SafetyCase']) {
        assert.equal(kindInfo(k).deriveOnly, true, k);
    }
    assert.equal(kindInfo('IBD').deriveOnly, undefined);
    const ok = buildCreateRequest(form({ kind: 'FaultTree', subject: 'Safety::FTA::FT', candidates: ['Safety::FTA::FT', 'Safety::SG'] }));
    assert.equal(ok.ok, true);
    assert.deepEqual(ok.request.fields, { diagramKind: 'FaultTree', subject: 'Safety::FTA::FT' });
    const blank = buildCreateRequest(form({ kind: 'SafetyCase', startFrom: 'blank', subject: '', candidates: [] }));
    assert.equal(blank.ok, false);
    assert.match(blank.error, /always derived/);
});

scenario('names follow the basic-name grammar', () => {
    assert.equal(validateName('PowerIBD'), null);
    assert.equal(validateName('_x1'), null);
    assert.match(validateName(''), /Give the diagram a name/);
    assert.match(validateName('   '), /Give the diagram a name/);
    assert.match(validateName('Power IBD'), /letters, digits and underscores/);
    assert.match(validateName('Power-IBD'), /letters, digits and underscores/);
    assert.match(validateName('1st'), /letters, digits and underscores/);
});

scenario('the default package is Diagrams when it exists, else the model root', () => {
    assert.equal(defaultPackage(['UAV', 'Diagrams', 'Views']), 'Diagrams');
    assert.equal(defaultPackage(['UAV', 'Views']), '');
    assert.equal(defaultPackage([]), '');
    assert.equal(defaultPackage(['UAV::Diagrams']), '', 'only a top-level package named Diagrams counts');
});

scenario('qualified names and tab ids', () => {
    assert.equal(joinQname('Diagrams', 'A'), 'Diagrams::A');
    assert.equal(joinQname('', 'A'), 'A');
    assert.equal(joinQname('UAV::Views', 'A'), 'UAV::Views::A');
    assert.equal(tabId('UAV::Views::A'), 'UAV/Views/A');
});

scenario('a derived diagram is a Diagram with diagramKind and subject, no shapes', () => {
    const r = buildCreateRequest(form({}));
    assert.equal(r.ok, true);
    assert.deepEqual(r.request, {
        qname: 'Diagrams::PowerIBD',
        type: 'Diagram',
        fields: { diagramKind: 'IBD', subject: 'UAV::Power::PowerSystem' },
    });
    assert.equal('shapes' in r.request.fields, false, 'no shapes: the subject selects the derived source');
    assert.equal(r.tabId, 'Diagrams/PowerIBD');
    assert.equal(r.displayName, 'PowerIBD');
    assert.equal(r.kind, 'IBD');
});

scenario('a derived diagram needs a subject, and it must be a suggestion', () => {
    const none = buildCreateRequest(form({ subject: '  ' }));
    assert.equal(none.ok, false);
    assert.match(none.error, /needs a subject/);
    const wrong = buildCreateRequest(form({ subject: 'UAV::Power' }));
    assert.equal(wrong.ok, false);
    assert.match(wrong.error, /not a part or item/);
    assert.match(wrong.error, /PartDef, Part, ItemDef, Item/);
});

scenario('a blank diagram is an empty manifest with an optional subject', () => {
    const r = buildCreateRequest(form({ startFrom: 'blank', subject: '' }));
    assert.equal(r.ok, true);
    assert.deepEqual(r.request.fields, { diagramKind: 'IBD', shapes: {} });
    const withSubject = buildCreateRequest(form({ startFrom: 'blank' }));
    assert.deepEqual(withSubject.request.fields, {
        diagramKind: 'IBD',
        subject: 'UAV::Power::PowerSystem',
        shapes: {},
    });
    const bad = buildCreateRequest(form({ startFrom: 'blank', subject: 'Nope' }));
    assert.equal(bad.ok, false);
});

scenario('the model root and nested packages build the right qualified name', () => {
    assert.equal(buildCreateRequest(form({ pkg: '' })).request.qname, 'PowerIBD');
    assert.equal(buildCreateRequest(form({ pkg: 'UAV::Views' })).request.qname, 'UAV::Views::PowerIBD');
    assert.equal(buildCreateRequest(form({ pkg: 'UAV::Views' })).tabId, 'UAV/Views/PowerIBD');
});

scenario('a bad name or kind is refused before any request is built', () => {
    assert.equal(buildCreateRequest(form({ name: 'a b' })).ok, false);
    const kind = buildCreateRequest(form({ kind: 'Mermaid' }));
    assert.equal(kind.ok, false);
    assert.match(kind.error, /BDD, IBD, StateMachine/);
});

console.log(`new-diagram: ok (${scenarios} scenarios)`);
