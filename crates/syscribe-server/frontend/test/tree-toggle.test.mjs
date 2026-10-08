// `npm test` (model browser tree): `static/js/tree-toggle.js`, the one handler
// behind a package row's ▶/▼ arrow. Driven with a fake row and a recording
// `ajax`, in the order the bug showed: the old markup split the toggle between
// an inline `onclick` and an htmx `hx-trigger` filter reading the state the
// click had just changed, so the first click loaded nothing and the second
// loaded the children it had just cleared.

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const require = createRequire(import.meta.url);
const { toggleTreeNode, OPEN, CLOSED } = require(path.join(root, 'static', 'js', 'tree-toggle.js'));

function row(url) {
    const children = { innerHTML: '<div>stale</div>', textContent: '' };
    const toggle = { dataset: { url }, textContent: CLOSED, parentElement: { nextElementSibling: children } };
    return { toggle, children };
}
function recorder(result) {
    const calls = [];
    const ajax = (method, url, opts) => {
        calls.push({ method, url, target: opts.target, swap: opts.swap });
        return result;
    };
    return { calls, ajax };
}

let scenarios = 0;
function scenario(name, fn) {
    fn();
    scenarios += 1;
    console.log(`  ok - ${name}`);
}

scenario('the first click opens the row and requests its children straight away', () => {
    const { toggle, children } = row('/ui/tree?parent=UAV');
    const { calls, ajax } = recorder(Promise.resolve());
    toggleTreeNode(toggle, ajax);
    assert.equal(toggle.textContent, OPEN);
    assert.equal(toggle.dataset.open, 'true');
    assert.deepEqual(calls, [{ method: 'GET', url: '/ui/tree?parent=UAV', target: children, swap: 'innerHTML' }]);
});

scenario('the second click collapses the row, clears it and requests nothing', () => {
    const { toggle, children } = row('/ui/tree?parent=UAV');
    const { calls, ajax } = recorder(Promise.resolve());
    toggleTreeNode(toggle, ajax);
    calls.length = 0;
    toggleTreeNode(toggle, ajax);
    assert.equal(toggle.textContent, CLOSED);
    assert.equal(toggle.dataset.open, 'false');
    assert.equal(children.innerHTML, '');
    assert.equal(calls.length, 0, 'closing never requests');
});

scenario('every open requests again, so a reopened folder is fresh', () => {
    const { toggle } = row('/ui/tree?parent=UAV');
    const { calls, ajax } = recorder(Promise.resolve());
    for (let i = 0; i < 5; i++) {
        toggleTreeNode(toggle, ajax);
    }
    assert.equal(calls.length, 3, 'open, close, open, close, open: three requests');
    assert.equal(toggle.dataset.open, 'true');
    assert.equal(toggle.textContent, OPEN);
});

scenario('the arrow and the state always agree', () => {
    const { toggle } = row('/ui/tree?parent=A');
    const { ajax } = recorder(Promise.resolve());
    for (let i = 0; i < 6; i++) {
        toggleTreeNode(toggle, ajax);
        assert.equal(toggle.textContent, toggle.dataset.open === 'true' ? OPEN : CLOSED);
    }
});

scenario('a failed load puts the row back to closed instead of an open arrow over nothing', async () => {
    const { toggle, children } = row('/ui/tree?parent=A');
    const { ajax } = recorder(Promise.reject(new Error('boom')));
    toggleTreeNode(toggle, ajax);
    await new Promise(r => setTimeout(r, 0));
    assert.equal(toggle.dataset.open, 'false');
    assert.equal(toggle.textContent, CLOSED);
    assert.match(children.textContent, /Could not load/);
});

scenario('the page loads the handler and the template calls it without a competing htmx trigger', () => {
    const base = readFileSync(path.join(root, 'templates', 'base.html'), 'utf8');
    const tree = readFileSync(path.join(root, 'templates', 'tree_items.html'), 'utf8');
    assert.ok(base.includes('/static/js/tree-toggle.js'), 'base.html loads tree-toggle.js');
    const toggleSpans = tree.match(/<span class="tree-toggle"[^>]*>/g) ?? [];
    assert.ok(toggleSpans.some(s => s.includes('onclick="toggleTreeNode(this)"')), 'a package row calls toggleTreeNode');
    for (const s of toggleSpans) {
        assert.ok(!s.includes('hx-trigger') && !s.includes('hx-get'), `no htmx trigger racing the handler: ${s}`);
    }
});

// The async scenario above was registered synchronously; let it settle before the summary.
await new Promise(r => setTimeout(r, 10));
console.log(`tree-toggle: ok (${scenarios} scenarios)`);
