// The outlines of the safety diagram symbols (GH #223): fault-tree gates and
// events, the attack-tree step and the GSN node shapes. A pure function of the
// node kind and its box — no sprotty, no DOM — so `test/safety-shape.test.mjs`
// can pin the geometry. It mirrors `vis::shape::shape_of` in Rust path for
// path, so the editor and the static SVG export draw the same symbol.

export type Outline = { type: 'rect'; rx: number } | { type: 'ellipse' } | { type: 'path'; d: string };

export type Extra =
    | { type: 'stroke'; d: string }
    | { type: 'circle'; cx: number; cy: number; r: number }
    | { type: 'diamond'; d: string }
    | { type: 'letter'; x: number; y: number; text: string };

export interface SafetySymbol {
    outline: Outline;
    extras: Extra[];
}

/** A number as `vis::shape::n` prints it: two decimals, no trailing `.0`. */
function n(v: number): string {
    const r = Math.round(v * 100) / 100;
    return String(r);
}

function polygon(pts: Array<[number, number]>): string {
    return pts.map(([x, y], i) => `${i === 0 ? 'M' : 'L'} ${n(x)},${n(y)} `).join('') + 'Z';
}

/** The OR-gate shield: pointed top, bowed sides, concave bottom. */
function orPath(w: number, h: number): string {
    return (
        `M ${n(0)},${n(h)} Q ${n(w / 2)},${n(0.72 * h)} ${n(w)},${n(h)} ` +
        `C ${n(w)},${n(0.5 * h)} ${n(0.7 * w)},${n(0.12 * h)} ${n(w / 2)},${n(0)} ` +
        `C ${n(0.3 * w)},${n(0.12 * h)} ${n(0)},${n(0.5 * h)} ${n(0)},${n(h)} Z`
    );
}

/** The symbol of a safety-diagram node kind in a `w`×`h` box at the origin,
 * or `undefined` for every other kind (drawn as a plain box). */
export function safetySymbol(kind: string | undefined, w: number, h: number): SafetySymbol | undefined {
    switch (kind) {
        case 'gate-and':
            return {
                outline: { type: 'path', d: `M ${n(0)},${n(h)} L ${n(0)},${n(0.5 * h)} A ${n(w / 2)},${n(0.5 * h)} 0 0 1 ${n(w)},${n(0.5 * h)} L ${n(w)},${n(h)} Z` },
                extras: [],
            };
        case 'gate-or':
            return { outline: { type: 'path', d: orPath(w, h) }, extras: [] };
        case 'gate-xor':
            return {
                outline: { type: 'path', d: orPath(w, h) },
                extras: [{ type: 'stroke', d: `M ${n(0)},${n(0.86 * h)} Q ${n(w / 2)},${n(0.58 * h)} ${n(w)},${n(0.86 * h)}` }],
            };
        case 'gate-not':
            return { outline: { type: 'rect', rx: 8 }, extras: [{ type: 'circle', cx: w / 2, cy: h, r: 5 }] };
        case 'gate-inhibit':
            return {
                outline: {
                    type: 'path',
                    d: polygon([
                        [0.12 * w, 0],
                        [0.88 * w, 0],
                        [w, h / 2],
                        [0.88 * w, h],
                        [0.12 * w, h],
                        [0, h / 2],
                    ]),
                },
                extras: [],
            };
        case 'event-basic':
        case 'solution':
            return { outline: { type: 'ellipse' }, extras: [] };
        case 'event-undeveloped':
            return {
                outline: {
                    type: 'path',
                    d: polygon([
                        [h / 2, 0],
                        [w - h / 2, 0],
                        [w, h / 2],
                        [w - h / 2, h],
                        [h / 2, h],
                        [0, h / 2],
                    ]),
                },
                extras: [],
            };
        case 'event-house':
            return {
                outline: {
                    type: 'path',
                    d: polygon([
                        [0, 0.28 * h],
                        [w / 2, 0],
                        [w, 0.28 * h],
                        [w, h],
                        [0, h],
                    ]),
                },
                extras: [],
            };
        case 'step':
            return { outline: { type: 'rect', rx: 3 }, extras: [] };
        case 'goal':
            return { outline: { type: 'rect', rx: 0 }, extras: [] };
        case 'undeveloped-goal': {
            const cx = w / 2;
            const cy = h + 8;
            const r = 8;
            return {
                outline: { type: 'rect', rx: 0 },
                extras: [
                    {
                        type: 'diamond',
                        d: polygon([
                            [cx, cy - r],
                            [cx + r, cy],
                            [cx, cy + r],
                            [cx - r, cy],
                        ]),
                    },
                ],
            };
        }
        case 'strategy':
            return {
                outline: {
                    type: 'path',
                    d: polygon([
                        [0.1 * w, 0],
                        [w, 0],
                        [0.9 * w, h],
                        [0, h],
                    ]),
                },
                extras: [],
            };
        case 'context':
            return { outline: { type: 'rect', rx: h / 2 }, extras: [] };
        case 'justification':
            return { outline: { type: 'ellipse' }, extras: [{ type: 'letter', x: w - 4, y: h + 11, text: 'J' }] };
        case 'assumption':
            return { outline: { type: 'ellipse' }, extras: [{ type: 'letter', x: w - 4, y: h + 11, text: 'A' }] };
        default:
            return undefined;
    }
}
