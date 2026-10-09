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
    | { type: 'letter'; x: number; y: number; text: string }
    | { type: 'word'; x: number; y: number; text: string };

export interface SafetySymbol {
    outline: Outline;
    extras: Extra[];
}

/** A number as `vis::shape::n` prints it: two decimals, no trailing `.0`. */
function n(v: number): string {
    const r = Math.round(v * 100) / 100;
    return String(r);
}

/** Width and height of a gate symbol, the stub joining it to its description
 * box, and the undeveloped-goal strip (`vis::shape`). */
export const GATE_W = 56;
export const GATE_H = 48;
export const STUB = 10;
export const UNDEVELOPED_STRIP = 18;
const GLYPH_GATE = STUB + GATE_H;

function polygon(pts: Array<[number, number]>): string {
    return pts.map(([x, y], i) => `${i === 0 ? 'M' : 'L'} ${n(x)},${n(y)} `).join('') + 'Z';
}

/** The OR-gate shield: pointed top, bowed sides, concave bottom. */
function orPath(x: number, y: number, w: number, h: number): string {
    return (
        `M ${n(x)},${n(y + h)} Q ${n(x + w / 2)},${n(y + 0.72 * h)} ${n(x + w)},${n(y + h)} ` +
        `C ${n(x + w)},${n(y + 0.5 * h)} ${n(x + 0.7 * w)},${n(y + 0.12 * h)} ${n(x + w / 2)},${n(y)} ` +
        `C ${n(x + 0.3 * w)},${n(y + 0.12 * h)} ${n(x)},${n(y + 0.5 * h)} ${n(x)},${n(y + h)} Z`
    );
}

function rectPath(x: number, y: number, w: number, h: number): string {
    return polygon([
        [x, y],
        [x + w, y],
        [x + w, y + h],
        [x, y + h],
    ]);
}

function circlePath(cx: number, cy: number, r: number): string {
    return `M ${n(cx - r)},${n(cy)} A ${n(r)},${n(r)} 0 1 0 ${n(cx + r)},${n(cy)} A ${n(r)},${n(r)} 0 1 0 ${n(cx - r)},${n(cy)} Z`;
}

function eventSymbol(kind: string | undefined): [number, number] | undefined {
    switch (kind) {
        case 'event-basic':
            return [28, 28];
        case 'event-undeveloped':
            return [40, 26];
        case 'event-house':
            return [34, 30];
        default:
            return undefined;
    }
}

function isGate(kind: string | undefined): boolean {
    return kind === 'gate-and' || kind === 'gate-or' || kind === 'gate-xor' || kind === 'gate-not' || kind === 'gate-inhibit';
}

/** The gate glyph with its top-left at `(gx, gy)`: outline, then the extras
 * drawn over it (`vis::shape::gate_glyph`). */
function gateGlyph(kind: string, gx: number, gy: number): { d: string; extras: Extra[] } {
    const w = GATE_W;
    const h = GATE_H;
    const cx = gx + w / 2;
    const word = (text: string, f: number): Extra => ({ type: 'word', x: cx, y: gy + f * h, text });
    switch (kind) {
        case 'gate-and':
            return {
                d: `M ${n(gx)},${n(gy + h)} L ${n(gx)},${n(gy + 0.5 * h)} A ${n(w / 2)},${n(0.5 * h)} 0 0 1 ${n(gx + w)},${n(gy + 0.5 * h)} L ${n(gx + w)},${n(gy + h)} Z`,
                extras: [word('AND', 0.78)],
            };
        case 'gate-or':
            return { d: orPath(gx, gy, w, h), extras: [word('OR', 0.72)] };
        case 'gate-xor': {
            const hh = h - 6;
            return {
                d: orPath(gx, gy, w, hh),
                extras: [{ type: 'stroke', d: `M ${n(gx)},${n(gy + h)} Q ${n(cx)},${n(gy + 0.72 * hh + 6)} ${n(gx + w)},${n(gy + h)}` }, word('XOR', 0.62)],
            };
        }
        case 'gate-not':
            return {
                d: polygon([
                    [cx, gy + 10],
                    [gx + w, gy + h],
                    [gx, gy + h],
                ]),
                extras: [{ type: 'circle', cx, cy: gy + 5, r: 5 }, word('NOT', 0.93)],
            };
        default:
            return {
                d: polygon([
                    [gx + 0.2 * w, gy],
                    [gx + 0.8 * w, gy],
                    [gx + w, gy + h / 2],
                    [gx + 0.8 * w, gy + h],
                    [gx + 0.2 * w, gy + h],
                    [gx, gy + h / 2],
                ]),
                extras: [word('INH', 0.58)],
            };
    }
}

/** The symbol of a safety-diagram node kind in a `w`×`h` box at the origin,
 * or `undefined` for every other kind (drawn as a plain box). A gate or event
 * is a description box with its symbol hung beneath it. */
export function safetySymbol(kind: string | undefined, w: number, h: number): SafetySymbol | undefined {
    const cx = w / 2;
    if (kind && isGate(kind)) {
        const rh = h - GLYPH_GATE;
        const g = gateGlyph(kind, cx - GATE_W / 2, rh + STUB);
        return {
            outline: { type: 'path', d: `${rectPath(0, 0, w, rh)} ${g.d}` },
            extras: [{ type: 'stroke', d: `M ${n(cx)},${n(rh)} L ${n(cx)},${n(rh + STUB)}` }, ...g.extras],
        };
    }
    const ev = eventSymbol(kind);
    if (ev) {
        const [sw, sh] = ev;
        const rh = h - STUB - sh;
        const sx = cx - sw / 2;
        const sy = rh + STUB;
        const sym =
            kind === 'event-basic'
                ? circlePath(cx, sy + sh / 2, sh / 2)
                : kind === 'event-undeveloped'
                  ? polygon([
                        [cx, sy],
                        [sx + sw, sy + sh / 2],
                        [cx, sy + sh],
                        [sx, sy + sh / 2],
                    ])
                  : polygon([
                        [sx, sy + 0.4 * sh],
                        [cx, sy],
                        [sx + sw, sy + 0.4 * sh],
                        [sx + sw, sy + sh],
                        [sx, sy + sh],
                    ]);
        return {
            outline: { type: 'path', d: `${rectPath(0, 0, w, rh)} ${sym}` },
            extras: [{ type: 'stroke', d: `M ${n(cx)},${n(rh)} L ${n(cx)},${n(rh + STUB)}` }],
        };
    }
    switch (kind) {
        case 'solution':
            return { outline: { type: 'ellipse' }, extras: [] };
        case 'step':
            return { outline: { type: 'rect', rx: 3 }, extras: [] };
        case 'goal':
            return { outline: { type: 'rect', rx: 0 }, extras: [] };
        case 'undeveloped-goal': {
            const rh = h - UNDEVELOPED_STRIP;
            const cy = rh + UNDEVELOPED_STRIP / 2;
            const r = 8;
            return {
                outline: { type: 'path', d: rectPath(0, 0, w, rh) },
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
            return { outline: { type: 'ellipse' }, extras: [{ type: 'letter', x: w - 9, y: h - 2, text: 'J' }] };
        case 'assumption':
            return { outline: { type: 'ellipse' }, extras: [{ type: 'letter', x: w - 9, y: h - 2, text: 'A' }] };
        default:
            return undefined;
    }
}
