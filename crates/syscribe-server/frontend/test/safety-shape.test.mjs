// `npm test` (GH #223): the safety diagram symbols in `src/safety-shape.ts`
// and the label-placement predicate in `src/layout.ts`. The literal path
// strings below are pinned identically by `vis::shape`'s Rust tests, so the
// editor and the static SVG export cannot drift apart. Bundled with the local
// esbuild into `test/.build/` (gitignored); no DOM needed.

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..');

async function load(entry, out) {
    const outfile = path.join(here, '.build', out);
    execFileSync(
        path.join(root, 'node_modules', '.bin', 'esbuild'),
        [path.join(root, 'src', entry), '--bundle', '--format=esm', '--platform=node', '--log-level=warning', `--outfile=${outfile}`],
        { stdio: 'inherit' },
    );
    return import(pathToFileURL(outfile).href);
}

const { safetySymbol } = await load('safety-shape.ts', 'safety-shape.mjs');
const { isSymbolKind } = await load('layout.ts', 'layout-symbols.mjs');

let scenarios = 0;
function scenario(name, fn) {
    fn();
    scenarios += 1;
    console.log(`  ok - ${name}`);
}

const KINDS = [
    'gate-and', 'gate-or', 'gate-xor', 'gate-not', 'gate-inhibit',
    'event-basic', 'event-undeveloped', 'event-house', 'step',
    'goal', 'undeveloped-goal', 'strategy', 'solution', 'context', 'justification', 'assumption',
];

scenario('every safety kind has a symbol and a plain block has none', () => {
    for (const k of KINDS) {
        assert.ok(safetySymbol(k, 120, 40), k);
    }
    assert.equal(safetySymbol('block', 120, 40), undefined);
    assert.equal(safetySymbol('feature', 120, 40), undefined);
    assert.equal(safetySymbol(undefined, 120, 40), undefined);
});

scenario('gate outlines are the paths vis::shape draws', () => {
    assert.deepEqual(safetySymbol('gate-or', 100, 50).outline, {
        type: 'path',
        d: 'M 0,50 Q 50,36 100,50 C 100,25 70,6 50,0 C 30,6 0,25 0,50 Z',
    });
    assert.deepEqual(safetySymbol('gate-and', 100, 50).outline, {
        type: 'path',
        d: 'M 0,50 L 0,25 A 50,25 0 0 1 100,25 L 100,50 Z',
    });
    const xor = safetySymbol('gate-xor', 100, 50);
    assert.deepEqual(xor.outline, safetySymbol('gate-or', 100, 50).outline, 'XOR is the OR shield');
    assert.deepEqual(xor.extras, [{ type: 'stroke', d: 'M 0,43 Q 50,29 100,43' }]);
    assert.deepEqual(safetySymbol('gate-not', 100, 50).extras, [{ type: 'circle', cx: 50, cy: 50, r: 5 }]);
    assert.equal(
        safetySymbol('gate-inhibit', 100, 50).outline.d,
        'M 12,0 L 88,0 L 100,25 L 88,50 L 12,50 L 0,25 Z',
    );
});

scenario('event and GSN symbols', () => {
    assert.deepEqual(safetySymbol('event-basic', 100, 50).outline, { type: 'ellipse' });
    assert.equal(
        safetySymbol('event-undeveloped', 100, 50).outline.d,
        'M 25,0 L 75,0 L 100,25 L 75,50 L 25,50 L 0,25 Z',
    );
    assert.equal(safetySymbol('event-house', 100, 50).outline.d, 'M 0,14 L 50,0 L 100,14 L 100,50 L 0,50 Z');
    assert.equal(safetySymbol('strategy', 100, 50).outline.d, 'M 10,0 L 100,0 L 90,50 L 0,50 Z');
    assert.deepEqual(safetySymbol('context', 100, 50).outline, { type: 'rect', rx: 25 });
    const undeveloped = safetySymbol('undeveloped-goal', 100, 50);
    assert.deepEqual(undeveloped.outline, { type: 'rect', rx: 0 });
    assert.deepEqual(undeveloped.extras, [{ type: 'diamond', d: 'M 50,50 L 58,58 L 50,66 L 42,58 Z' }]);
    assert.deepEqual(safetySymbol('assumption', 100, 50).extras, [{ type: 'letter', x: 96, y: 61, text: 'A' }]);
    assert.deepEqual(safetySymbol('justification', 100, 50).extras, [{ type: 'letter', x: 96, y: 61, text: 'J' }]);
});

scenario('the label stack is centred in the symbols with a narrower outline only', () => {
    for (const k of ['gate-and', 'gate-or', 'gate-xor', 'gate-not', 'gate-inhibit', 'event-basic', 'event-undeveloped', 'event-house', 'strategy', 'solution', 'context', 'justification', 'assumption']) {
        assert.equal(isSymbolKind(k), true, k);
    }
    for (const k of ['step', 'goal', 'undeveloped-goal', 'block', 'feature', 'state', undefined]) {
        assert.equal(isSymbolKind(k), false, String(k));
    }
});

console.log(`safety-shape: ok (${scenarios} scenarios)`);
