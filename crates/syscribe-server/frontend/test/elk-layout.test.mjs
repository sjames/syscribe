// `npm test` (REQ-TRS-VIS-014): run the bundled elkjs the way the browser
// does (`src/layout.ts`'s `SyscribeLayoutConfigurator`) over a fixture IBD —
// a boundary with its own port, two blocks with ports, a connection edge and
// a binding edge — and assert the two properties the picture depends on:
// no two sibling nodes overlap, and every port's centre sits on its parent's
// border. The ELK graph is hand-built with the same option values the
// configurator emits, so a change to those values is exercised here without
// a DOM. Node-compatible: elkjs's bundled build needs no worker or browser.
//
// Run with `npm test` from `crates/syscribe-server/frontend/`.

import assert from 'node:assert/strict';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const ELK = require('elkjs/lib/elk.bundled.js');

// --- the same option values `layout.ts` uses --------------------------------
const PORT_SIZE = 12;
const graphOptions = {
    'elk.algorithm': 'layered',
    'elk.direction': 'RIGHT',
    'elk.hierarchyHandling': 'INCLUDE_CHILDREN',
    'elk.edgeRouting': 'ORTHOGONAL',
    'elk.spacing.nodeNode': '40',
    'elk.layered.spacing.nodeNodeBetweenLayers': '60',
    'elk.spacing.edgeNode': '30',
    'elk.layered.spacing.edgeNodeBetweenLayers': '30',
    'elk.spacing.portPort': '16',
    'elk.spacing.labelLabel': '1',
    'elk.padding': '[top=20,left=20,bottom=20,right=20]',
};
function nodeOptions({ minW, minH, compound, fixedSides }) {
    const o = {
        'elk.nodeSize.constraints': 'NODE_LABELS PORTS PORT_LABELS MINIMUM_SIZE',
        'elk.nodeSize.minimum': `(${minW}, ${minH})`,
        'elk.nodeLabels.placement': compound ? '[H_LEFT, V_TOP, INSIDE]' : '[H_CENTER, V_TOP, INSIDE]',
        'elk.nodeLabels.padding': '[top=4,left=8,bottom=4,right=8]',
        'elk.portLabels.placement': 'OUTSIDE',
        'elk.portConstraints': fixedSides ? 'FIXED_SIDE' : 'FREE',
    };
    if (compound) {
        o['elk.padding'] = '[top=44,left=20,bottom=20,right=20]';
    }
    return o;
}
function port(id, text, side) {
    return {
        id,
        width: PORT_SIZE,
        height: PORT_SIZE,
        layoutOptions: { 'elk.port.borderOffset': String(-PORT_SIZE / 2), 'elk.port.side': side },
        labels: [{ id: `${id}-label`, text, width: text.length * 5, height: 10 }],
    };
}
function block(id, name, ports) {
    return {
        id,
        layoutOptions: nodeOptions({ minW: 120, minH: 40, compound: false, fixedSides: true }),
        labels: [
            { id: `${id}-stereotype`, text: '«part»', width: 32, height: 10 },
            { id: `${id}-label`, text: name, width: name.length * 7, height: 14 },
        ],
        ports,
    };
}

// --- fixture: PowerSystem IBD --------------------------------------------------
const fixture = {
    id: 'sysml-diagram',
    layoutOptions: graphOptions,
    children: [
        {
            id: 's-boundary',
            layoutOptions: nodeOptions({ minW: 120, minH: 40, compound: true, fixedSides: true }),
            labels: [
                { id: 's-boundary-stereotype', text: '«part def»', width: 50, height: 10 },
                { id: 's-boundary-label', text: 'PowerSystem', width: 80, height: 14 },
            ],
            ports: [port('s-main-pout', 'mainPowerOut', 'EAST')],
            children: [
                block('s-battery', 'battery : BatteryPack', [port('s-battery-pout', 'powerOut', 'EAST')]),
                block('s-pdu', 'pdu : PowerDistributionUnit', [
                    port('s-pdu-pin', 'powerIn', 'WEST'),
                    port('s-pdu-pout', 'powerOut', 'EAST'),
                ]),
            ],
        },
    ],
    edges: [
        { id: 'e-batt-pdu', sources: ['s-battery-pout'], targets: ['s-pdu-pin'] },
        { id: 'e-pdu-bind', sources: ['s-pdu-pout'], targets: ['s-main-pout'] },
    ],
};

// --- checks ----------------------------------------------------------------------
function overlaps(a, b) {
    return a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;
}

function checkNode(node, path, stats) {
    const kids = node.children ?? [];
    for (let i = 0; i < kids.length; i++) {
        for (let j = i + 1; j < kids.length; j++) {
            assert.ok(
                !overlaps(kids[i], kids[j]),
                `${path}: siblings ${kids[i].id} and ${kids[j].id} overlap`,
            );
            stats.pairs++;
        }
    }
    for (const p of node.ports ?? []) {
        for (const k of ['x', 'y', 'width', 'height']) {
            assert.equal(typeof p[k], 'number', `${p.id}.${k} is set`);
        }
        const cx = p.x + p.width / 2;
        const cy = p.y + p.height / 2;
        const onVertical = Math.abs(cx) <= 1 || Math.abs(cx - node.width) <= 1;
        const onHorizontal = Math.abs(cy) <= 1 || Math.abs(cy - node.height) <= 1;
        assert.ok(
            onVertical || onHorizontal,
            `${p.id}: centre (${cx}, ${cy}) is not on the border of ${node.id} (${node.width}×${node.height})`,
        );
        // Port labels are placed OUTSIDE: the label box must not intersect the parent.
        for (const l of p.labels ?? []) {
            const box = { x: p.x + l.x, y: p.y + l.y, width: l.width, height: l.height };
            assert.ok(
                !overlaps(box, { x: 0, y: 0, width: node.width, height: node.height }),
                `${l.id}: port label lies inside ${node.id}`,
            );
        }
        stats.ports++;
    }
    for (const l of node.labels ?? []) {
        assert.ok(l.x >= 0 && l.y >= 0 && l.x + l.width <= node.width + 0.5 && l.y + l.height <= node.height + 0.5,
            `${l.id}: label does not fit inside ${node.id}`);
    }
    for (const c of kids) {
        assert.ok(
            c.x >= 0 && c.y >= 0 && c.x + c.width <= node.width + 0.5 && c.y + c.height <= node.height + 0.5,
            `${c.id} is not inside its parent ${node.id}`,
        );
        checkNode(c, `${path}/${c.id}`, stats);
    }
}

const elk = new ELK();
const result = await elk.layout(structuredClone(fixture));
const stats = { pairs: 0, ports: 0 };
checkNode(result, result.id, stats);
assert.ok(stats.ports === 4, `expected 4 ports checked, got ${stats.ports}`);
assert.ok(stats.pairs >= 1, 'expected at least one sibling pair checked');
for (const e of result.edges) {
    assert.ok(e.sections && e.sections.length > 0, `${e.id} was routed`);
}
console.log(`elk-layout: ok (${stats.ports} ports on borders, ${stats.pairs} sibling pairs disjoint, ${result.edges.length} edges routed)`);
