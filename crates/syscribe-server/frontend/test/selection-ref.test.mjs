// `npm test` (REQ-TRS-VIS-026): which element a selection means for the side
// panel — `src/selection-ref.ts`, bundled with the local esbuild into
// `test/.build/selection-ref.mjs` (gitignored); no DOM.

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..');
const outfile = path.join(here, '.build', 'selection-ref.mjs');
execFileSync(
    path.join(root, 'node_modules', '.bin', 'esbuild'),
    [path.join(root, 'src', 'selection-ref.ts'), '--bundle', '--format=esm', '--platform=node', '--log-level=warning', `--outfile=${outfile}`],
    { stdio: 'inherit' },
);
const { refForSelection, cardUrl } = await import(pathToFileURL(outfile).href);

let scenarios = 0;
function scenario(name, fn) {
    fn();
    scenarios += 1;
    console.log(`  ok - ${name}`);
}

const port = { type: 'port', id: 'p-out', ref: 'A::B::engine::powerOut', kind: 'port', name: 'powerOut', children: [{ type: 'label', id: 'p-out-label', text: 'powerOut' }] };
const block = { type: 'node', id: 'n-engine', ref: 'A::B::engine', kind: 'block', name: 'engine', children: [{ type: 'label', id: 'n-engine-label', text: 'engine' }, port] };
const boundary = { type: 'node', id: 'n-b', ref: 'A::B', kind: 'boundary', name: 'B', children: [{ type: 'label', id: 'n-b-label', text: 'B' }, block] };
const noRef = { type: 'node', id: 'n-note', kind: 'note', name: 'note', children: [] };
const model = {
    id: 'g',
    type: 'graph',
    children: [
        boundary,
        noRef,
        { type: 'edge', id: 'e1', sourceId: 'p-out', targetId: 'n-b', kind: 'flow', ref: 'A::B::Link' },
        { type: 'edge', id: 'e2', sourceId: 'p-out', targetId: 'n-b', kind: 'flow' },
    ],
};

scenario('one selected shape means its ref, at any nesting depth', () => {
    assert.equal(refForSelection(model, ['n-b']), 'A::B');
    assert.equal(refForSelection(model, ['n-engine']), 'A::B::engine');
    assert.equal(refForSelection(model, ['p-out']), 'A::B::engine::powerOut');
});

scenario('one selected edge means its ref, and an edge without one means nothing', () => {
    assert.equal(refForSelection(model, ['e1']), 'A::B::Link');
    assert.equal(refForSelection(model, ['e2']), null);
});

scenario('nothing selected, several selected or a non-element selected change nothing', () => {
    assert.equal(refForSelection(model, []), null);
    assert.equal(refForSelection(model, ['n-b', 'n-engine']), null, 'several: the panel stays as it is');
    assert.equal(refForSelection(model, ['n-engine-label']), null, 'a label is not a model element');
    assert.equal(refForSelection(model, ['missing']), null);
    assert.equal(refForSelection(model, ['n-note']), null, 'a shape with no ref');
});

scenario('a selection can be any iterable, such as a Set', () => {
    assert.equal(refForSelection(model, new Set(['n-engine'])), 'A::B::engine');
});

scenario('the card URL uses / for :: and escapes each segment', () => {
    assert.equal(cardUrl('A::B::engine'), '/ui/element-card/A/B/engine');
    assert.equal(cardUrl('Reqs::REQ-1'), '/ui/element-card/Reqs/REQ-1');
    assert.equal(cardUrl('A::b c::d?'), '/ui/element-card/A/b%20c/d%3F');
});

console.log(`selection-ref: ok (${scenarios} scenarios)`);
