// `npm test` (REQ-TRS-VIS-027): the pure helpers of the planning dashboard in
// static/js/planning-core.js (boardUrl, age, mergePeople); no DOM.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const core = createRequire(import.meta.url)(path.join(here, '..', '..', 'static', 'js', 'planning-core.js'));

let n = 0;
function scenario(name, fn) { fn(); n += 1; console.log(`  ok - ${name}`); }

scenario('the board URL carries the filters and escapes them', () => {
    assert.equal(core.boardUrl('', false), '/ui/planning/board');
    assert.equal(core.boardUrl('agent-7', false), '/ui/planning/board?who=agent-7');
    assert.equal(core.boardUrl('Alice Archer', true), '/ui/planning/board?who=Alice%20Archer&done=1');
    assert.equal(core.boardUrl('', true), '/ui/planning/board?done=1');
});

scenario('ages are short and tick forward', () => {
    const t = Date.parse('2026-10-08T10:00:00Z');
    assert.equal(core.age('2026-10-08T10:00:00Z', t + 2000), 'now');
    assert.equal(core.age('2026-10-08T10:00:00Z', t + 30000), '30s');
    assert.equal(core.age('2026-10-08T10:00:00Z', t + 180000), '3m');
    assert.equal(core.age('2026-10-08T10:00:00Z', t + 2 * 3600000), '2h');
    assert.equal(core.age('2026-10-08T10:00:00Z', t + 5 * 86400000), '5d');
    assert.equal(core.age('2026-10-08T10:00:00Z', t - 60000), 'now', 'a clock a little ahead is now, not negative');
    assert.equal(core.age('not a date', t), '');
});

scenario('the people list keeps the current filter even when it has no items', () => {
    assert.deepEqual(core.mergePeople([''], ['b', 'a', 'a'], 'zed'), ['', 'a', 'b', 'zed']);
    assert.deepEqual(core.mergePeople(['', 'a'], [], ''), ['', 'a']);
});

console.log(`planning-core: ok (${n} scenarios)`);
