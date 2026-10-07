// `npm test` (REQ-TRS-VIS-017): server sizes are authoritative. A fixture in
// the shape of `GET /api/diagrams/model/{qname}` whose nodes, ports, labels
// and compartments carry `size` (what a server with Rust-side text metrics
// sends) is run through `src/layout.ts`'s preparation, then through
// sprotty-elk's `ElkLayoutEngine` with the real `SyscribeLayoutConfigurator`
// and `SyscribeLayoutProcessor`, and the ELK graph that reaches elkjs is
// checked to carry exactly those widths/heights — plus the options that keep
// ELK from growing a sized leaf for its text. One unsized block in the same
// fixture covers the older-server fallback (ELK sizes it from its labels).
// The full layout then runs and the sizes must survive it unchanged.
//
// Like `connect-rules.test.mjs`, the TypeScript is bundled first with the
// local esbuild into `test/.build/` (gitignored); sprotty-elk's engine and
// sprotty-protocol's index need no DOM.
//
// Run with `npm test` from `crates/syscribe-server/frontend/` (after `npm ci`).

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

// --- build the module under test -----------------------------------------------
const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..');
const esbuild = path.join(root, 'node_modules', '.bin', 'esbuild');
const outfile = path.join(here, '.build', 'layout.mjs');
execFileSync(
    esbuild,
    [path.join(root, 'src', 'layout.ts'), '--bundle', '--format=esm', '--platform=node', '--log-level=warning', `--outfile=${outfile}`],
    { stdio: 'inherit' },
);
const { prepareForLayout, LayoutState, SyscribeLayoutConfigurator, SyscribeLayoutProcessor, PORT_SIZE } = await import(
    pathToFileURL(outfile).href
);
const require = createRequire(import.meta.url);
const ELK = require('elkjs/lib/elk.bundled.js');
const { ElkLayoutEngine } = require('sprotty-elk/lib/elk-layout');
const { SModelIndex } = require('sprotty-protocol/lib/utils/model-utils');

// --- fixture builders (the wire shape, with `size` as the server sends it) --------
const sz = (width, height) => ({ width, height });
function label(id, text, size) {
    const l = { type: 'label', id, text };
    if (size) {
        l.size = size;
    }
    return l;
}
function port(id, ref, name, direction, side, size, labelSize) {
    const p = { type: 'port', id, ref, kind: 'port', name, direction, side, children: [label(`${id}-label`, name, labelSize)] };
    if (size) {
        p.size = size;
    }
    return p;
}
function block(id, ref, name, size, children, extra = {}) {
    const n = { type: 'node', id, ref, kind: 'block', elementType: 'Part', stereotype: 'part', name, children, ...extra };
    if (size) {
        n.size = size;
    }
    return n;
}

// Sizes chosen so none is a default the client would pick on its own.
const SIZES = {
    boundary: sz(93, 31), boundaryLabel: sz(77, 15), mainOut: sz(12, 12), mainOutLabel: sz(61, 11),
    battery: sz(163, 47), batteryLabel: sz(131, 15), battOut: sz(16, 16), battOutLabel: sz(43, 11),
    pdu: sz(207, 73), pduLabel: sz(181, 15), pduAttrs: sz(207, 27), pduLine0: sz(71, 12), pduLine1: sz(83, 12),
    pduIn: sz(12, 12), pduInLabel: sz(39, 11), pduOut: sz(12, 12), pduOutLabel: sz(43, 11),
};

const model = {
    type: 'graph',
    id: 'sysml-diagram',
    qualifiedName: 'Diagrams::PowerSystemDerivedIBD',
    diagramKind: 'IBD',
    subject: 'UAV::Power::PowerSystem',
    derived: true,
    layoutOptions: { 'elk.algorithm': 'layered', 'elk.direction': 'RIGHT', 'elk.hierarchyHandling': 'INCLUDE_CHILDREN', 'syscribe.reversedEdgeKinds': [] },
    pinned: [],
    children: [
        {
            type: 'node', id: 's-ps', ref: 'UAV::Power::PowerSystem', kind: 'boundary', elementType: 'PartDef', stereotype: 'part def', name: 'PowerSystem',
            size: SIZES.boundary,
            children: [
                label('s-ps-label', 'PowerSystem', SIZES.boundaryLabel),
                port('s-ps-mainpowerout', 'UAV::Power::PowerSystem::mainPowerOut', 'mainPowerOut', 'out', 'east', SIZES.mainOut, SIZES.mainOutLabel),
                block('s-battery', 'UAV::Power::PowerSystem::battery', 'battery : BatteryPack', SIZES.battery, [
                    label('s-battery-label', 'battery : BatteryPack', SIZES.batteryLabel),
                    port('s-battery-powerout', 'UAV::Power::PowerSystem::battery::powerOut', 'powerOut', 'out', 'east', SIZES.battOut, SIZES.battOutLabel),
                ]),
                block('s-pdu', 'UAV::Power::PowerSystem::pdu', 'pdu : PowerDistributionUnit', SIZES.pdu, [
                    label('s-pdu-label', 'pdu : PowerDistributionUnit', SIZES.pduLabel),
                    {
                        type: 'compartment', id: 's-pdu-attrs', kind: 'attributes', ref: 'UAV::Power::PowerSystem::pdu', name: 'attributes',
                        lines: ['maxCurrent : Real', 'busVoltage : Real'], size: SIZES.pduAttrs,
                        children: [label('s-pdu-attrs-line-0', 'maxCurrent : Real', SIZES.pduLine0), label('s-pdu-attrs-line-1', 'busVoltage : Real', SIZES.pduLine1)],
                    },
                    port('s-pdu-powerin', 'UAV::Power::PowerSystem::pdu::powerIn', 'powerIn', 'in', 'west', SIZES.pduIn, SIZES.pduInLabel),
                    port('s-pdu-powerout', 'UAV::Power::PowerSystem::pdu::powerOut', 'powerOut', 'out', 'east', SIZES.pduOut, SIZES.pduOutLabel),
                ]),
            ],
        },
        // An older server: no sizes anywhere (the port gets the client's 12×12).
        block('s-legacy', 'UAV::Power::Legacy', 'legacy : Gauge', undefined, [
            label('s-legacy-label', 'legacy : Gauge'),
            port('s-legacy-sig', 'UAV::Power::Legacy::sig', 'sig', 'in', 'west'),
        ], { banners: ['Safety'] }),
        { type: 'edge', id: 'e-batt-pdu', kind: 'connection', sourceId: 's-battery-powerout', targetId: 's-pdu-powerin', label: 'dc', style: { stroke: '#333', keyword: 'connect' } },
        { type: 'edge', id: 'e-pdu-main', kind: 'binding', sourceId: 's-pdu-powerout', targetId: 's-ps-mainpowerout', style: { stroke: '#333', keyword: '=' } },
        { type: 'edge', id: 'e-main-legacy', kind: 'flow', sourceId: 's-ps-mainpowerout', targetId: 's-legacy-sig' },
    ],
};

let checks = 0;
async function step(name, body) {
    await body();
    checks++;
    console.log(`  ok - ${name}`);
}
function find(parent, id) {
    for (const c of parent.children ?? []) {
        if (c.id === id) {
            return c;
        }
        const hit = find(c, id);
        if (hit) {
            return hit;
        }
    }
    return undefined;
}
function* elkShapes(node) {
    yield node;
    for (const l of node.labels ?? []) {
        yield l;
    }
    for (const p of node.ports ?? []) {
        yield p;
        for (const l of p.labels ?? []) {
            yield l;
        }
    }
    for (const e of node.edges ?? []) {
        for (const l of e.labels ?? []) {
            yield l;
        }
    }
    for (const c of node.children ?? []) {
        yield* elkShapes(c);
    }
}
const sizeOf = s => ({ width: s.width, height: s.height });

// --- 1. preparation copies the server size and anchors the vbox on it -------------
prepareForLayout(model);

await step('every sized element keeps its server size as serverSize', () => {
    const expect = {
        's-ps': SIZES.boundary, 's-ps-label': SIZES.boundaryLabel, 's-ps-mainpowerout': SIZES.mainOut, 's-ps-mainpowerout-label': SIZES.mainOutLabel,
        's-battery': SIZES.battery, 's-battery-label': SIZES.batteryLabel, 's-battery-powerout': SIZES.battOut, 's-battery-powerout-label': SIZES.battOutLabel,
        's-pdu': SIZES.pdu, 's-pdu-label': SIZES.pduLabel, 's-pdu-attrs': SIZES.pduAttrs, 's-pdu-attrs-line-0': SIZES.pduLine0, 's-pdu-attrs-line-1': SIZES.pduLine1,
        's-pdu-powerin': SIZES.pduIn, 's-pdu-powerin-label': SIZES.pduInLabel, 's-pdu-powerout': SIZES.pduOut, 's-pdu-powerout-label': SIZES.pduOutLabel,
    };
    for (const [id, size] of Object.entries(expect)) {
        const el = find(model, id);
        assert.ok(el, `${id} is in the tree`);
        assert.deepEqual(el.serverSize, size, `${id}.serverSize`);
        assert.deepEqual(sizeOf(el.size), size, `${id}.size untouched`);
    }
    // vbox containers with a server size are told not to grow around their children.
    assert.equal(find(model, 's-battery').layoutOptions.resizeContainer, false, 'sized node: resizeContainer false');
    assert.equal(find(model, 's-pdu-attrs').layoutOptions.resizeContainer, false, 'sized compartment: resizeContainer false');
    assert.equal(find(model, 's-battery').layout, 'vbox', 'the micro-layout still stacks the children');
    // The synthetic stereotype label is client-made and unsized.
    const st = find(model, 's-battery-stereotype');
    assert.ok(st && st.role === 'stereotype', 'stereotype label was added');
    assert.equal(st.serverSize, undefined);
});

await step('an unsized element is left to the measuring pass', () => {
    const legacy = find(model, 's-legacy');
    assert.equal(legacy.serverSize, undefined);
    assert.equal(legacy.size, undefined, 'no size invented for a node');
    assert.equal(legacy.layoutOptions.resizeContainer, true, 'unsized node: the vbox grows around its children');
    assert.equal(find(model, 's-legacy-label').serverSize, undefined);
    const sig = find(model, 's-legacy-sig');
    assert.deepEqual(sig.size, sz(PORT_SIZE, PORT_SIZE), 'an unsized port gets the client default');
    assert.equal(sig.serverSize, undefined, 'but that default is not a server size');
    // Edge keyword/label children are client-made and unsized.
    for (const id of ['e-batt-pdu-keyword', 'e-batt-pdu-label', 'e-pdu-main-keyword']) {
        const l = find(model, id);
        assert.ok(l, `${id} was added`);
        assert.equal(l.serverSize, undefined);
    }
});

// --- 2. the ELK graph carries exactly the server sizes ---------------------------------
const state = new LayoutState();
const configurator = new SyscribeLayoutConfigurator(state);
const processor = new SyscribeLayoutProcessor(state);
const engine = new ElkLayoutEngine(() => new ELK(), undefined, configurator, processor, processor);
const index = new SModelIndex();
index.add(model);
const elkGraph = engine.transformGraph(model, index);
processor.preprocess(elkGraph, model, index);

await step('the ELK input uses exactly the server widths/heights', () => {
    let sized = 0;
    for (const shape of elkShapes(elkGraph)) {
        const el = shape.id ? index.getById(shape.id) : undefined;
        if (!el || !el.serverSize) {
            continue;
        }
        assert.equal(shape.width, el.serverSize.width, `${shape.id}.width`);
        assert.equal(shape.height, el.serverSize.height, `${shape.id}.height`);
        sized++;
    }
    // 3 nodes + 4 ports + 3 node labels + 4 port labels (compartments are not ELK shapes).
    assert.equal(sized, 14, `sized ELK shapes: ${sized}`);
    const legacy = elkGraph.children.find(c => c.id === 's-legacy');
    assert.equal(legacy.width, undefined, 'an unsized node reaches ELK without a width');
    assert.equal(legacy.labels.find(l => l.id === 's-legacy-label').width, undefined, 'and its label without one');
    assert.deepEqual(sizeOf(legacy.ports[0]), sz(PORT_SIZE, PORT_SIZE), 'its port is the client default');
});

await step('a sized leaf is fixed by options too, a sized compound at least that big, an unsized leaf sized by ELK', () => {
    const ps = elkGraph.children.find(c => c.id === 's-ps');
    const battery = ps.children.find(c => c.id === 's-battery');
    const pdu = ps.children.find(c => c.id === 's-pdu');
    const legacy = elkGraph.children.find(c => c.id === 's-legacy');
    assert.equal(battery.layoutOptions['elk.nodeSize.constraints'], 'PORTS MINIMUM_SIZE', 'sized leaf: no NODE_LABELS/PORT_LABELS growth');
    assert.equal(battery.layoutOptions['elk.nodeSize.minimum'], '(163, 47)');
    assert.equal(pdu.layoutOptions['elk.nodeSize.constraints'], 'PORTS MINIMUM_SIZE');
    assert.equal(pdu.layoutOptions['elk.nodeSize.minimum'], '(207, 73)');
    assert.equal(ps.layoutOptions['elk.nodeSize.constraints'], 'NODE_LABELS PORTS PORT_LABELS MINIMUM_SIZE', 'compound: grows around its children');
    assert.equal(ps.layoutOptions['elk.nodeSize.minimum'], '(120, 40)', 'compound: the server size is a floor under the client minimum');
    assert.equal(legacy.layoutOptions['elk.nodeSize.constraints'], 'NODE_LABELS PORTS PORT_LABELS MINIMUM_SIZE', 'unsized leaf: as before');
    assert.equal(legacy.layoutOptions['elk.nodeSize.minimum'], '(120, 40)');
    // Ports sit centred on the border whatever their size.
    assert.equal(battery.ports[0].layoutOptions['elk.port.borderOffset'], '-8', '16×16 port');
    assert.equal(pdu.ports[0].layoutOptions['elk.port.borderOffset'], '-6', '12×12 port');
    assert.equal(legacy.ports[0].layoutOptions['elk.port.borderOffset'], '-6', 'default port');
});

// --- 3. the sizes survive the layout -------------------------------------------------------
function overlaps(a, b) {
    return a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;
}

await engine.layout(model, index);

await step('after layout every sized leaf, port and label still has its server size', () => {
    for (const id of ['s-battery', 's-pdu']) {
        assert.deepEqual(sizeOf(find(model, id).size), find(model, id).serverSize, `${id}.size`);
    }
    for (const id of ['s-ps-mainpowerout', 's-battery-powerout', 's-pdu-powerin', 's-pdu-powerout', 's-ps-label', 's-battery-label', 's-pdu-label', 's-pdu-powerin-label', 's-battery-powerout-label']) {
        assert.deepEqual(sizeOf(find(model, id).size), find(model, id).serverSize, `${id}.size`);
    }
    const ps = find(model, 's-ps');
    assert.ok(ps.size.width >= SIZES.boundary.width && ps.size.height >= SIZES.boundary.height, 'the compound grew around its children');
    assert.ok(ps.size.width > SIZES.pdu.width, 'and holds the pdu');
    // The unsized block was sized by ELK from its minimum (no measured labels here).
    const legacy = find(model, 's-legacy');
    assert.ok(legacy.size.width >= 120 && legacy.size.height >= 40, `legacy sized by ELK: ${JSON.stringify(legacy.size)}`);
});

await step('labels lie inside their sized node and ports on its border', () => {
    for (const id of ['s-battery', 's-pdu']) {
        const node = find(model, id);
        const box = { x: 0, y: 0, ...node.size };
        for (const c of node.children) {
            if (c.type === 'label' && c.serverSize) {
                assert.ok(c.position, `${c.id} placed`);
                assert.ok(
                    c.position.x >= 0 && c.position.y >= 0 && c.position.x + c.size.width <= box.width + 0.5 && c.position.y + c.size.height <= box.height + 0.5,
                    `${c.id} at ${JSON.stringify(c.position)} ${JSON.stringify(c.size)} fits ${id} ${JSON.stringify(node.size)}`,
                );
            } else if (c.type === 'port') {
                const cx = c.position.x + c.size.width / 2;
                const cy = c.position.y + c.size.height / 2;
                const onVertical = Math.abs(cx) <= 1 || Math.abs(cx - box.width) <= 1;
                const onHorizontal = Math.abs(cy) <= 1 || Math.abs(cy - box.height) <= 1;
                assert.ok(onVertical || onHorizontal, `${c.id} centre (${cx}, ${cy}) is on the border of ${id}`);
                const l = c.children[0];
                const lbox = { x: c.position.x + l.position.x, y: c.position.y + l.position.y, ...l.size };
                assert.ok(!overlaps(lbox, box), `${l.id} is outside ${id}`);
            }
        }
    }
    for (const e of model.children.filter(c => c.type === 'edge')) {
        assert.ok(e.routingPoints && e.routingPoints.length >= 2, `${e.id} routed`);
    }
});

await step('a second run over the same cached schema is identical', async () => {
    // What `DiagramEditor.activate` does on every tab switch: the cached
    // schema (now holding ELK's result) goes through the engine again.
    const snapshot = JSON.stringify(model);
    await engine.layout(model, index);
    assert.equal(JSON.stringify(model), snapshot, 'positions, sizes and routes are stable');
});

console.log(`server-sizes: ok (${checks} checks; ${model.children.length} root children laid out)`);
