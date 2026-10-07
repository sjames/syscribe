// `npm test` (REQ-TRS-VIS-008, TC-TRS-VIS-008): drive the pure connect rules
// in `src/connect-rules.ts` — `resolveConnectEnds`, `compatible`, `portChain`
// and `isDerivedDiagram` — over hand-built `DiagramModelSchema` fixtures, one
// assertion group per scenario of the qualification case. The source is
// TypeScript, so the script first bundles it with the local esbuild into
// `test/.build/connect-rules.mjs` (gitignored) and imports that; no DOM,
// sprotty or elkjs is needed because the rules only touch the schema.
//
// Run with `npm test` from `crates/syscribe-server/frontend/` (after `npm ci`).

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';

// --- build the module under test -----------------------------------------------
const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '..');
const esbuild = path.join(root, 'node_modules', '.bin', 'esbuild');
const outfile = path.join(here, '.build', 'connect-rules.mjs');
execFileSync(
    esbuild,
    [
        path.join(root, 'src', 'connect-rules.ts'),
        '--bundle',
        '--format=esm',
        '--platform=node',
        '--log-level=warning',
        `--outfile=${outfile}`,
    ],
    { stdio: 'inherit' },
);
const { resolveConnectEnds, compatible, portChain, isDerivedDiagram, refSlug } = await import(
    pathToFileURL(outfile).href
);

// --- fixture builders (the shape of `GET /api/diagrams/model/{qname}`) -----------
function port(id, ref, name, direction) {
    const p = { type: 'port', id, ref, kind: 'port', name, children: [] };
    if (direction !== undefined) {
        p.direction = direction;
    }
    return p;
}
function block(id, ref, name, ports) {
    return { type: 'node', id, ref, kind: 'block', name, children: ports };
}
function boundary(id, ref, name, children) {
    return { type: 'node', id, ref, kind: 'boundary', name, children };
}
function graph(subject, children) {
    const g = { type: 'graph', id: 'sysml-diagram', qualifiedName: 'A::IBD', diagramKind: 'ibd', layoutOptions: {}, pinned: [], children };
    if (subject !== undefined) {
        g.subject = subject;
    }
    return g;
}
/** A port's id for a derived diagram: `vis::derive`'s deterministic slug. */
function derivedPort(ref, name, direction) {
    return port(refSlug(ref), ref, name, direction);
}

// --- fixture: derived IBD of subject A::B ------------------------------------------
const SUBJECT = 'A::B';
const mainPowerOut = derivedPort('A::B::mainPowerOut', 'mainPowerOut', 'out');
const batteryPowerOut = derivedPort('A::B::battery::powerOut', 'powerOut', 'out');
const pduPowerIn = derivedPort('A::B::pdu::powerIn', 'powerIn', 'in');
const pduPowerOut = derivedPort('A::B::pdu::powerOut', 'powerOut', 'out');
const battery = block(refSlug('A::B::battery'), 'A::B::battery', 'battery : BatteryPack', [batteryPowerOut]);
const pdu = block(refSlug('A::B::pdu'), 'A::B::pdu', 'pdu : PowerDistributionUnit', [pduPowerIn, pduPowerOut]);
const derived = graph(SUBJECT, [boundary(refSlug(SUBJECT), SUBJECT, 'B', [mainPowerOut, battery, pdu])]);

let scenarios = 0;
function scenario(name, body) {
    body();
    scenarios++;
    console.log(`  ok - ${name}`);
}

// --- Scenario: port to port with compatible directions ------------------------------
scenario('port to port with compatible directions', () => {
    const r = resolveConnectEnds(batteryPowerOut, pduPowerIn);
    assert.equal(r.ok, true, 'out→in is accepted');
    assert.equal(r.source, batteryPowerOut);
    assert.equal(r.target, pduPowerIn);
    // in→out is just as valid: direction is about compatibility, not gesture order.
    const back = resolveConnectEnds(pduPowerIn, batteryPowerOut);
    assert.equal(back.ok, true, 'in→out is accepted');
    assert.equal(back.source, pduPowerIn);
    assert.equal(back.target, batteryPowerOut);
    // The chains relative to the subject are what POST /api/connections carries.
    assert.equal(portChain(derived, r.source, SUBJECT), 'battery.powerOut');
    assert.equal(portChain(derived, r.target, SUBJECT), 'pdu.powerIn');
    // `inout` and an undirected port go with anything, including a same-side port.
    const inout = port('p-io', 'A::B::x::io', 'io', 'inout');
    const undirected = port('p-u', 'A::B::x::u', 'u');
    assert.equal(compatible(inout, batteryPowerOut), true, 'inout goes with out');
    assert.equal(compatible(undirected, pduPowerIn), true, 'undirected goes with in');
    assert.equal(compatible(inout, undirected), true, 'inout goes with undirected');
    assert.equal(compatible(batteryPowerOut, pduPowerIn), true, 'out goes with in');
    assert.equal(compatible(batteryPowerOut, pduPowerOut), false, 'out does not go with out');
    assert.equal(compatible(pduPowerIn, pduPowerIn), false, 'in does not go with in');
    // A direction carried only by the style glyph counts the same way.
    const glyphOnly = { type: 'port', id: 'p-g', ref: 'A::B::x::g', kind: 'port', name: 'g', style: { glyph: 'out' } };
    assert.equal(compatible(glyphOnly, pduPowerOut), false, 'glyph out does not go with out');
    assert.equal(compatible(glyphOnly, pduPowerIn), true, 'glyph out goes with in');
});

// --- Scenario: a block stands in for its single compatible port ----------------------
scenario('a block stands in for its single compatible port', () => {
    // battery (one out port) → pdu (one in, one out): the only compatible pair is powerOut→powerIn.
    const r = resolveConnectEnds(battery, pdu);
    assert.equal(r.ok, true, `block→block is accepted: ${r.reason ?? ''}`);
    assert.equal(r.source, batteryPowerOut, 'source is the battery out port');
    assert.equal(r.target, pduPowerIn, 'target is the pdu in port');
    // A block on one end only.
    const halfA = resolveConnectEnds(batteryPowerOut, pdu);
    assert.equal(halfA.ok, true);
    assert.equal(halfA.source, batteryPowerOut);
    assert.equal(halfA.target, pduPowerIn);
    const halfB = resolveConnectEnds(battery, pduPowerIn);
    assert.equal(halfB.ok, true);
    assert.equal(halfB.source, batteryPowerOut);
    assert.equal(halfB.target, pduPowerIn);
    // The reverse gesture pdu → battery also has exactly one compatible pair (powerIn→powerOut).
    const rev = resolveConnectEnds(pdu, battery);
    assert.equal(rev.ok, true);
    assert.equal(rev.source, pduPowerIn);
    assert.equal(rev.target, batteryPowerOut);
    // Only *direct* port children stand in: a port nested under a sub-block does not count.
    const inner = block('b-inner', 'A::B::outer::inner', 'inner', [port('p-deep', 'A::B::outer::inner::deep', 'deep', 'out')]);
    const outer = block('b-outer', 'A::B::outer', 'outer', [inner]);
    const nested = resolveConnectEnds(outer, pduPowerIn);
    assert.equal(nested.ok, false, 'a port owned by a sub-block does not stand in for the outer block');
    assert.match(nested.reason, /'outer' has no ports to connect from/);
});

// --- Scenario: two ports of the same direction are refused ------------------------------
scenario('two ports of the same direction are refused', () => {
    const r = resolveConnectEnds(batteryPowerOut, pduPowerOut);
    assert.equal(r.ok, false, 'out→out is refused');
    assert.equal(r.source, undefined, 'nothing to write: no ends resolved');
    assert.match(r.reason, /'powerOut' \(out\) to 'powerOut' \(out\)/, 'the toast names both directions');
    assert.match(r.reason, /same direction/);
    const inIn = resolveConnectEnds(pduPowerIn, port('p-in2', 'A::B::sink::dataIn', 'dataIn', 'in'));
    assert.equal(inIn.ok, false, 'in→in is refused');
    assert.match(inIn.reason, /'powerIn' \(in\) to 'dataIn' \(in\)/);
    // Block → block with no compatible pair at all (two in-only blocks) is refused too.
    const sinkA = block('b-sa', 'A::B::sinkA', 'sinkA', [port('p-sa', 'A::B::sinkA::in', 'in', 'in')]);
    const sinkB = block('b-sb', 'A::B::sinkB', 'sinkB', [port('p-sb', 'A::B::sinkB::in', 'in', 'in')]);
    const none = resolveConnectEnds(sinkA, sinkB);
    assert.equal(none.ok, false);
    assert.match(none.reason, /No compatible port pair between 'sinkA' and 'sinkB'/);
    assert.match(none.reason, /an out port must meet an in port/);
});

// --- Scenario: an ambiguous block pair is refused ---------------------------------------
scenario('an ambiguous block pair is refused', () => {
    const src = block('b-src', 'A::B::src', 'src', [
        port('p-s1', 'A::B::src::out1', 'out1', 'out'),
        port('p-s2', 'A::B::src::out2', 'out2', 'out'),
    ]);
    const tgt = block('b-tgt', 'A::B::tgt', 'tgt', [
        port('p-t1', 'A::B::tgt::in1', 'in1', 'in'),
        port('p-t2', 'A::B::tgt::in2', 'in2', 'in'),
    ]);
    const r = resolveConnectEnds(src, tgt);
    assert.equal(r.ok, false, 'four compatible pairs is ambiguous');
    assert.match(r.reason, /^Ambiguous: 4 compatible port pairs between 'src' and 'tgt'/);
    for (const pair of ['out1→in1', 'out1→in2', 'out2→in1', 'out2→in2']) {
        assert.ok(r.reason.includes(pair), `the toast lists ${pair}`);
    }
    assert.match(r.reason, /connect the two ports directly$/);
    // Two candidates is already ambiguous: one out port against a block with two in ports.
    const two = resolveConnectEnds(src.children[0], tgt);
    assert.equal(two.ok, false);
    assert.match(two.reason, /^Ambiguous: 2 compatible port pairs between 'out1' and 'tgt' \(out1→in1, out1→in2\)/);
    // Connecting the two ports directly resolves it.
    const direct = resolveConnectEnds(src.children[0], tgt.children[1]);
    assert.equal(direct.ok, true);
    assert.equal(direct.source.name, 'out1');
    assert.equal(direct.target.name, 'in2');
});

// --- Scenario: a block with no ports is refused -------------------------------------------
scenario('a block with no ports is refused', () => {
    const bare = block('b-bare', 'A::B::bare', 'bare', []);
    const asSource = resolveConnectEnds(bare, pduPowerIn);
    assert.equal(asSource.ok, false);
    assert.equal(asSource.reason, "'bare' has no ports to connect from — connect from a port");
    const asTarget = resolveConnectEnds(batteryPowerOut, bare);
    assert.equal(asTarget.ok, false);
    assert.equal(asTarget.reason, "'bare' has no ports to connect to — connect to a port");
    const both = resolveConnectEnds(bare, block('b-bare2', 'A::B::bare2', 'bare2', []));
    assert.equal(both.ok, false, 'two bare blocks: the source is reported first');
    assert.match(both.reason, /^'bare' has no ports to connect from/);
    // A block whose only children are labels/compartments has no ports either.
    const labelled = {
        type: 'node', id: 'b-lab', ref: 'A::B::lab', kind: 'block', name: 'lab',
        children: [
            { type: 'label', id: 'b-lab-label', text: 'lab' },
            { type: 'compartment', id: 'b-lab-attrs', kind: 'attributes', ref: 'A::B::lab', name: 'attributes', lines: ['mass : Real'] },
        ],
    };
    const lab = resolveConnectEnds(labelled, pduPowerIn);
    assert.equal(lab.ok, false);
    assert.match(lab.reason, /'lab' has no ports to connect from/);
    // A node with no name falls back to its ref in the toast.
    const anon = { type: 'node', id: 'b-anon', ref: 'A::B::anon', kind: 'block', name: '', children: [] };
    assert.match(resolveConnectEnds(anon, pduPowerIn).reason, /^'A::B::anon' has no ports/);
});

// --- Scenario: the connections entry is a dotted chain relative to the subject ------------
scenario('the connections entry is a dotted chain relative to the subject', () => {
    // A port under a block of the subject: `block.port`.
    assert.equal(portChain(derived, batteryPowerOut, SUBJECT), 'battery.powerOut');
    assert.equal(portChain(derived, pduPowerIn, SUBJECT), 'pdu.powerIn');
    // The subject's own (boundary) port: its bare name.
    assert.equal(portChain(derived, mainPowerOut, SUBJECT), 'mainPowerOut');
    // Deeper nesting spells every segment under the owner.
    const deep = derivedPort('A::B::pdu::fuse::out', 'out', 'out');
    assert.equal(portChain(derived, deep, SUBJECT), 'pdu.fuse.out');

    // A manifest port whose ref is not spelled under the subject (the port def's
    // own qname, say) is rebuilt from the enclosing blocks' names, skipping the boundary.
    const libOut = port('batt-out', 'Lib::BatteryPack::powerOut', 'powerOut', 'out');
    const libIn = port('pdu-in', 'Lib::PowerDistributionUnit::powerIn', 'powerIn', 'in');
    const boundaryOwn = port('main-out', 'Lib::PowerSystem::mainPowerOut', 'mainPowerOut', 'out');
    const manifest = graph(SUBJECT, [
        boundary('frame', SUBJECT, 'B', [
            boundaryOwn,
            block('batt', 'A::B::battery', 'battery', [libOut]),
            block('pdu', 'A::B::pdu', 'pdu', [
                block('fuse', 'A::B::pdu::fuse', 'fuse', [libIn]),
            ]),
        ]),
    ]);
    assert.equal(portChain(manifest, libOut, SUBJECT), 'battery.powerOut', 'rebuilt from the enclosing block');
    assert.equal(portChain(manifest, libIn, SUBJECT), 'pdu.fuse.powerIn', 'rebuilt through every enclosing block');
    assert.equal(portChain(manifest, boundaryOwn, SUBJECT), 'mainPowerOut', 'the boundary is skipped');
    // A manifest block whose ref is itself the subject is skipped like the boundary
    // (it *is* the owner), and a root-level port with no enclosing block is its bare name.
    const flat = graph(SUBJECT, [
        block('owner', SUBJECT, 'B', [block('batt', 'A::B::battery', 'battery', [libOut])]),
        port('loose', 'Lib::Loose::sig', 'sig', 'in'),
    ]);
    assert.equal(portChain(flat, libOut, SUBJECT), 'battery.powerOut');
    assert.equal(portChain(flat, flat.children[1], SUBJECT), 'sig');
});

// --- Scenario: a derived diagram sends no diagram sync block ----------------------------
scenario('a derived diagram sends no diagram sync block', () => {
    // `isDerivedDiagram` is the switch the editor uses to drop the `diagram:` block:
    // a subject and every root shape id equal to its ref's slug.
    assert.equal(refSlug('A::B'), 's-a-b');
    assert.equal(refSlug('Vehicle::Power_System::battery'), 's-vehicle-power-system-battery');
    assert.equal(derived.children[0].id, 's-a-b');
    assert.equal(isDerivedDiagram(derived), true, 'derived: subject + slug ids');
    // Several roots, all slugs, with edges in between (edges are not shapes and are ignored).
    const multi = graph('A::C', [
        block(refSlug('A::C::x'), 'A::C::x', 'x', []),
        block(refSlug('A::C::y'), 'A::C::y', 'y', []),
        { type: 'edge', id: 'e-1', kind: 'connection', sourceId: 's-a-c-x', targetId: 's-a-c-y' },
    ]);
    assert.equal(isDerivedDiagram(multi), true);
    // A root-level port counts as a root shape too.
    const withPort = graph('A::C', [derivedPort('A::C::sig', 'sig', 'in')]);
    assert.equal(isDerivedDiagram(withPort), true);
    // No subject: never derived, even with slug ids.
    const noSubject = graph(undefined, [boundary(refSlug(SUBJECT), SUBJECT, 'B', [])]);
    assert.equal(isDerivedDiagram(noSubject), false, 'no subject → not derived');
    // A subject but no shapes at all: nothing to be derived from.
    assert.equal(isDerivedDiagram(graph(SUBJECT, [])), false, 'no root shapes → not derived');
});

// --- Scenario: a manifest diagram syncs the edge ------------------------------------------
scenario('a manifest diagram syncs the edge', () => {
    // Author-chosen ids mean a manifest exists: the editor then sends the `diagram:` block.
    const manifest = graph(SUBJECT, [boundary('frame', SUBJECT, 'B', [mainPowerOut, battery, pdu])]);
    assert.equal(isDerivedDiagram(manifest), false, 'author id on the root → manifest');
    // One author-chosen id among slug ids is enough.
    const mixed = graph('A::C', [
        block(refSlug('A::C::x'), 'A::C::x', 'x', []),
        block('y', 'A::C::y', 'y', []),
    ]);
    assert.equal(isDerivedDiagram(mixed), false, 'any non-slug root id → manifest');
    // A slug-shaped id that is not *this* ref's slug is still an author id.
    const wrongSlug = graph('A::C', [block(refSlug('A::C::z'), 'A::C::x', 'x', [])]);
    assert.equal(isDerivedDiagram(wrongSlug), false);
    // The resolved ends on a manifest diagram are the same port shapes, whose ids
    // are what the edge's sourceId/targetId carry in the sync block.
    const r = resolveConnectEnds(battery, pdu);
    assert.equal(r.ok, true);
    assert.equal(r.source.id, refSlug('A::B::battery::powerOut'));
    assert.equal(r.target.id, refSlug('A::B::pdu::powerIn'));
});

console.log(`connect-rules: ok (${scenarios} scenarios)`);
