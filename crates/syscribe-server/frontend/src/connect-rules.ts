// Port-aware connect rules (`REQ-TRS-VIS-008`, design §6.3) — pure functions
// over the schema, kept apart from `editor.ts` so they are easy to reason
// about: which two ports a connect gesture joins, why a gesture is refused,
// and how a port is spelled as the dotted chain `connections:` expects.

import {
    childrenOf,
    containerOf,
    DiagramModelSchema,
    isPortSchema,
    SysmlNodeChildSchema,
    SysmlPortSchema,
    SysmlShapeSchema,
} from './types';
import { isContainerKind } from './layout';

export type ConnectEnds =
    | { ok: true; source: SysmlPortSchema; target: SysmlPortSchema }
    | { ok: false; reason: string };

type Direction = 'in' | 'out' | 'inout' | 'none';

function directionOf(port: SysmlPortSchema): Direction {
    const d = port.direction ?? port.style?.glyph;
    return d === 'in' || d === 'out' || d === 'inout' ? d : 'none';
}

/** `out`→`in` and `in`→`out`; `inout` and an undirected port go with anything. */
export function compatible(a: SysmlPortSchema, b: SysmlPortSchema): boolean {
    const da = directionOf(a);
    const db = directionOf(b);
    if (da === 'none' || db === 'none' || da === 'inout' || db === 'inout') {
        return true;
    }
    return da !== db;
}

function directPorts(shape: SysmlShapeSchema): SysmlPortSchema[] {
    return (childrenOf(shape) as SysmlNodeChildSchema[]).filter(isPortSchema);
}

function describe(shape: SysmlShapeSchema): string {
    return `'${shape.name || shape.ref}'`;
}

/** Decide the two ports a gesture from `source` to `target` joins. Either end
 * may be a port (used as is) or a block (which must own exactly one port
 * compatible with the other side); anything else is refused with the reason
 * the toast shows. */
export function resolveConnectEnds(source: SysmlShapeSchema, target: SysmlShapeSchema): ConnectEnds {
    const sourceIsPort = isPortSchema(source);
    const targetIsPort = isPortSchema(target);

    if (sourceIsPort && targetIsPort) {
        if (!compatible(source, target)) {
            return {
                ok: false,
                reason: `Cannot connect ${describe(source)} (${directionOf(source)}) to ${describe(target)} (${directionOf(target)}): both ports have the same direction`,
            };
        }
        return { ok: true, source, target };
    }

    const sourcePorts = sourceIsPort ? [source] : directPorts(source);
    const targetPorts = targetIsPort ? [target] : directPorts(target);
    if (sourcePorts.length === 0) {
        return { ok: false, reason: `${describe(source)} has no ports to connect from — connect from a port` };
    }
    if (targetPorts.length === 0) {
        return { ok: false, reason: `${describe(target)} has no ports to connect to — connect to a port` };
    }
    const pairs: [SysmlPortSchema, SysmlPortSchema][] = [];
    for (const s of sourcePorts) {
        for (const t of targetPorts) {
            if (s !== t && compatible(s, t)) {
                pairs.push([s, t]);
            }
        }
    }
    if (pairs.length === 0) {
        return {
            ok: false,
            reason: `No compatible port pair between ${describe(source)} and ${describe(target)} (an out port must meet an in port)`,
        };
    }
    if (pairs.length > 1) {
        const list = pairs.map(([s, t]) => `${s.name}→${t.name}`).join(', ');
        return {
            ok: false,
            reason: `Ambiguous: ${pairs.length} compatible port pairs between ${describe(source)} and ${describe(target)} (${list}) — connect the two ports directly`,
        };
    }
    return { ok: true, source: pairs[0][0], target: pairs[0][1] };
}

function lastSegment(qname: string): string {
    const parts = qname.split('::');
    return parts[parts.length - 1] || qname;
}

/** The dotted feature chain `add_connection` resolves relative to the owner
 * (`crates/syscribe-server/src/routes/mutate.rs`): a port `A::B::battery::powerOut`
 * owned by `A::B` is `battery.powerOut`; the owner's own port
 * `A::B::mainPowerOut` is `mainPowerOut`. When the port's `ref` is not
 * spelled under the owner (a manifest with its own ids), the chain is
 * rebuilt from the enclosing blocks' names, skipping the boundary. */
export function portChain(model: DiagramModelSchema, port: SysmlPortSchema, ownerQname: string): string {
    const prefix = ownerQname + '::';
    if (port.ref.startsWith(prefix)) {
        return port.ref.slice(prefix.length).replace(/::/g, '.');
    }
    const parts: string[] = [lastSegment(port.ref)];
    let cur = containerOf(model, port.id);
    while (cur && cur !== model) {
        const shape = cur as SysmlShapeSchema;
        if (shape.ref !== ownerQname && !isContainerKind(shape.kind)) {
            parts.unshift(lastSegment(shape.ref));
        }
        cur = containerOf(model, shape.id);
    }
    return parts.join('.');
}

/** Whether the diagram is derived from its `subject:` (no manifest to sync).
 * The graph root carries the answer as `derived` (`vis::sprotty`), which wins
 * whenever it is present. An older server sends no flag; then a derived
 * view is recognised by its shape ids: all `s-<ref slug>` (`vis::derive`'s
 * deterministic ids), whereas a manifest uses the author's keys, so a root
 * shape whose id is not its ref's slug means a manifest exists. */
export function isDerivedDiagram(model: DiagramModelSchema): boolean {
    if (typeof model.derived === 'boolean') {
        return model.derived;
    }
    if (!model.subject) {
        return false;
    }
    const roots = (model.children as SysmlNodeChildSchema[]).filter(
        (c): c is SysmlShapeSchema => c.type === 'node' || c.type === 'port',
    );
    return roots.length > 0 && roots.every(r => r.id === refSlug(r.ref));
}

export function refSlug(ref: string): string {
    return 's-' + ref.replace(/[^A-Za-z0-9]+/g, '-').replace(/^-|-$/g, '').toLowerCase();
}
