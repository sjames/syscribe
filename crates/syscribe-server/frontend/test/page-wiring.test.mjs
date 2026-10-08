// `npm test` (REQ-TRS-DE-004, REQ-TRS-VIS-023): the page's own wiring, which no
// browser-less test otherwise sees. `static/css/app.css` hides
// `#sprotty-viewport` (`display: none`), so `templates/base.html` must show it
// with an explicit display value: assigning `''` merely removes the inline
// style and leaves the stylesheet's `none` in force, so every diagram tab opens
// to a blank canvas (the bug this test pins; present from the editor's first
// commit until the New diagram work found it).

import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const css = readFileSync(path.join(root, 'static', 'css', 'app.css'), 'utf8');
const base = readFileSync(path.join(root, 'templates', 'base.html'), 'utf8');
const index = readFileSync(path.join(root, 'templates', 'index.html'), 'utf8');

let checks = 0;
function check(name, fn) {
    fn();
    checks += 1;
    console.log(`  ok - ${name}`);
}

// Ids the stylesheet hides by default (`#id { ... display: none ... }`).
const hiddenByCss = new Set(
    [...css.matchAll(/(^|\n)#([A-Za-z][\w-]*)\s*\{([^}]*)\}/g)]
        .filter(m => /display:\s*none/.test(m[3]))
        .map(m => m[2]),
);

// `const bar = document.getElementById('canvas-tabs')` style aliases.
const aliases = new Map(
    [...base.matchAll(/(?:const|let|var)\s+(\w+)\s*=\s*document\.getElementById\('([^']+)'\)/g)].map(m => [m[1], m[2]]),
);
// Every `<target>.style.display = '<value>'` in the page script, with <target>
// resolved to an element id (direct `getElementById('id')` or an alias).
const assignments = [
    ...base.matchAll(/(?:document\.getElementById\('([^']+)'\)|\b(\w+))\.style\.display\s*=\s*'([^']*)'/g),
].map(m => ({ id: m[1] ?? aliases.get(m[2]), target: m[1] ?? m[2], value: m[3] }));

check('the page assigns display values to the elements the stylesheet hides', () => {
    const ids = new Set(assignments.map(a => a.id));
    assert.ok(hiddenByCss.has('sprotty-viewport'), 'app.css hides #sprotty-viewport by default');
    assert.ok(hiddenByCss.has('canvas-tabs'), 'app.css hides #canvas-tabs by default');
    assert.ok(ids.has('sprotty-viewport') && ids.has('canvas-tabs'), 'base.html shows and hides both');
});

check("no element the stylesheet hides is 'shown' by clearing its inline display", () => {
    const bad = assignments.filter(a => a.id && hiddenByCss.has(a.id) && a.value === '');
    assert.deepEqual(
        bad.map(a => `#${a.id} (via ${a.target})`),
        [],
        "an empty display removes the inline style and the stylesheet's `display: none` wins, so the element stays hidden; use 'block' or 'flex'",
    );
});

check('the viewport, host and toast exist in the page the script drives', () => {
    for (const id of ['sprotty-viewport', 'sprotty-host', 'sprotty-toast', 'canvas-tabs-inner', 'mermaid-view']) {
        assert.ok(index.includes(`id="${id}"`) || base.includes(`id="${id}"`), `#${id} is in the templates`);
    }
});

check('the element side panel ids the script reads are present and the panel uses the hidden attribute', () => {
    for (const id of ['element-panel', 'element-panel-body', 'ep-close']) {
        assert.ok(index.includes(`id="${id}"`), `#${id} is in index.html`);
    }
    assert.match(index, /<aside id="element-panel" hidden>/, 'hidden by attribute, which no inline display can fight');
    assert.ok(/#element-panel\[hidden\]\s*\{\s*display:\s*none/.test(css), 'the stylesheet keeps [hidden] hidden');
});

check('the feature page ids the script reads are present in its template and the bundle is built', () => {
    const tpl = readFileSync(path.join(root, 'templates', 'features.html'), 'utf8');
    for (const id of ['fm-host', 'fm-canvas', 'fm-search', 'fm-collapse', 'fm-expand', 'fm-fit', 'fm-banner', 'fm-summary', 'fm-selected', 'fm-live', 'fm-empty']) {
        assert.ok(tpl.includes(`id="${id}"`), `#${id} is in features.html`);
    }
    assert.ok(existsSync(path.join(root, 'static', 'js', 'feature-model.js')), 'static/js/feature-model.js is built');
    assert.ok(tpl.includes('/static/js/feature-model.js'), 'the page loads the bundle');
});

check('the Add existing element button and picker ids the script reads are all present', () => {
    for (const id of ['add-existing-dialog', 'ae-form', 'ae-ref', 'ae-results', 'ae-count', 'ae-error', 'ae-add', 'ae-cancel']) {
        assert.ok(index.includes(`id="${id}"`), `#${id} is in index.html`);
    }
    assert.ok(index.includes('window.DiagramEditor.addExisting()'), 'the toolbar button calls addExisting');
});

check('the New diagram control and dialog ids the script reads are all present', () => {
    for (const id of ['new-diagram-dialog', 'nd-form', 'nd-name', 'nd-kind', 'nd-subject', 'nd-subjects', 'nd-subject-label', 'nd-subject-hint', 'nd-package', 'nd-error', 'nd-create', 'nd-cancel']) {
        assert.ok(index.includes(`id="${id}"`), `#${id} is in index.html`);
    }
    assert.ok(index.includes('window.NewDiagram.open()'), 'the sidebar button opens the dialog');
    assert.equal((index.match(/name="nd-start"/g) ?? []).length, 2, 'derive and blank radios');
});

console.log(`page-wiring: ok (${checks} checks)`);
