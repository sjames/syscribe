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
const { isSymbolKind, treeRoute, isTreeEdgeKind } = await load('layout.ts', 'layout-symbols.mjs');

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

const RECT_32 = 'M 0,0 L 100,0 L 100,32 L 0,32 Z';

scenario('gate outlines are the paths vis::shape draws (a 100x90 box)', () => {
    const d = k => safetySymbol(k, 100, 90).outline.d;
    assert.equal(d('gate-and'), `${RECT_32} M 22,90 L 22,66 A 28,24 0 0 1 78,66 L 78,90 Z`);
    assert.equal(d('gate-or'), `${RECT_32} M 22,90 Q 50,76.56 78,90 C 78,66 61.2,47.76 50,42 C 38.8,47.76 22,66 22,90 Z`);
    assert.equal(d('gate-inhibit'), `${RECT_32} M 33.2,42 L 66.8,42 L 78,66 L 66.8,90 L 33.2,90 L 22,66 Z`);
    assert.equal(d('gate-not'), `${RECT_32} M 50,52 L 78,90 L 22,90 Z`);
    const xor = safetySymbol('gate-xor', 100, 90);
    assert.deepEqual(xor.extras.slice(0, 2), [
        { type: 'stroke', d: 'M 50,32 L 50,42' },
        { type: 'stroke', d: 'M 22,90 Q 50,78.24 78,90' },
    ]);
    assert.deepEqual(xor.extras[2], { type: 'word', x: 50, y: 42 + 0.62 * 48, text: 'XOR' });
    const not = safetySymbol('gate-not', 100, 90);
    assert.ok(not.extras.some(e => e.type === 'circle' && e.cx === 50 && e.cy === 47 && e.r === 5));
});

scenario('event and GSN symbols', () => {
    const d = k => safetySymbol(k, 100, 90).outline.d;
    assert.equal(d('event-basic'), 'M 0,0 L 100,0 L 100,52 L 0,52 Z M 36,76 A 14,14 0 1 0 64,76 A 14,14 0 1 0 36,76 Z');
    assert.equal(d('event-undeveloped'), 'M 0,0 L 100,0 L 100,54 L 0,54 Z M 50,64 L 70,77 L 50,90 L 30,77 Z');
    assert.equal(d('event-house'), 'M 0,0 L 100,0 L 100,50 L 0,50 Z M 33,72 L 50,60 L 67,72 L 67,90 L 33,90 Z');
    assert.equal(d('strategy'), 'M 10,0 L 100,0 L 90,90 L 0,90 Z');
    assert.deepEqual(safetySymbol('context', 100, 50).outline, { type: 'rect', rx: 25 });
    const undeveloped = safetySymbol('undeveloped-goal', 100, 90);
    assert.equal(undeveloped.outline.d, 'M 0,0 L 100,0 L 100,72 L 0,72 Z');
    assert.deepEqual(undeveloped.extras, [{ type: 'diamond', d: 'M 50,73 L 58,81 L 50,89 L 42,81 Z' }]);
    assert.deepEqual(safetySymbol('assumption', 100, 50).extras, [{ type: 'letter', x: 91, y: 48, text: 'A' }]);
    assert.deepEqual(safetySymbol('justification', 100, 50).extras, [{ type: 'letter', x: 91, y: 48, text: 'J' }]);
});

scenario('the label stack is centred only in the GSN symbols with a narrower outline', () => {
    for (const k of ['strategy', 'solution', 'context', 'justification', 'assumption']) {
        assert.equal(isSymbolKind(k), true, k);
    }
    for (const k of ['gate-and', 'gate-or', 'event-basic', 'event-house', 'step', 'goal', 'undeveloped-goal', 'block', 'feature', 'state', undefined]) {
        assert.equal(isSymbolKind(k), false, String(k));
    }
});

scenario('tree edges are drawn as a bus unless a node is in the way', () => {
    const s = { x: 100, y: 0, w: 100, h: 50 };
    const t = { x: 0, y: 150, w: 100, h: 50 };
    assert.deepEqual(treeRoute(s, t, 20, []), [
        { x: 150, y: 50 },
        { x: 150, y: 70 },
        { x: 50, y: 70 },
        { x: 50, y: 150 },
    ]);
    assert.deepEqual(treeRoute(s, { x: 100, y: 150, w: 100, h: 50 }, 20, []), [{ x: 150, y: 50 }, { x: 150, y: 150 }]);
    assert.equal(treeRoute(s, t, 20, [{ x: 0, y: 60, w: 300, h: 20 }]), undefined, 'a node on the channel');
    assert.equal(treeRoute(s, { x: 0, y: 60, w: 100, h: 50 }, 20, []), undefined, 'a child that is not clearly below');
    for (const k of ['input', 'criticalPath', 'supportedBy', 'inContextOf']) {
        assert.equal(isTreeEdgeKind(k), true, k);
    }
    assert.equal(isTreeEdgeKind('flow'), false);
});

console.log(`safety-shape: ok (${scenarios} scenarios)`);
