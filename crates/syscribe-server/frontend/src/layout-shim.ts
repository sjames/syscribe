// Phase-0 placement shim (`ADR-SYS-VIS-001` §9 Phase 0). The diagram-model
// endpoint carries geometry only for pinned nodes (`REQ-TRS-VIS-006`), and
// the real layout engine — ELK through `sprotty-elk`, `REQ-TRS-VIS-007` —
// lands in Phase 2. Until then this module gives every unpinned element a
// deterministic, parent-relative position and a default size by kind, so the
// nested graph is drawn legibly instead of piling every child on its parent's
// origin. It lives in the client (not in `vis::sprotty`) so the server
// contract stays honest: a `position` in the JSON always means a human pinned
// it, and `model.pinned` stays the one source of that fact.
//
// Everything here is idempotent: an element that already has a `position`
// (pinned, previously placed, or just dragged) is left alone, and a `size`
// is only filled in where missing. Phase 2 deletes this file.

import {
    childrenOf,
    DiagramModelSchema,
    isCompartmentSchema,
    isEdgeSchema,
    isLabelSchema,
    isPortSchema,
    isShapeSchema,
    SysmlLabelSchema,
    SysmlNodeChildSchema,
    SysmlShapeSchema,
} from './types';

export const PORT_SIZE = 12;
const PADDING = 24;
const HEADER = 36;
const GAP = 40;
const LINE_HEIGHT = 14;
const MAX_PER_ROW = 4;
const MAX_PER_COLUMN = 3;

/** Default size by IR node role; Phase 2 replaces these with measured sizes. */
export function defaultSize(kind: string): { width: number; height: number } {
    switch (kind) {
        case 'port':
            return { width: PORT_SIZE, height: PORT_SIZE };
        case 'boundary':
        case 'system-boundary':
        case 'swimlane':
            return { width: 400, height: 260 };
        case 'requirement':
        case 'testcase':
            return { width: 180, height: 70 };
        case 'state':
        case 'usecase':
            return { width: 140, height: 50 };
        case 'note':
            return { width: 160, height: 60 };
        case 'actor':
            return { width: 60, height: 90 };
        case 'lifeline':
            return { width: 120, height: 300 };
        case 'initial':
        case 'final':
        case 'choice':
        case 'history':
            return { width: 24, height: 24 };
        case 'fragment':
            return { width: 300, height: 160 };
        default:
            return { width: 160, height: 50 };
    }
}

type Sized = { position?: { x: number; y: number }; size?: { width: number; height: number } };

function ensureSize(el: Sized, kind: string): { width: number; height: number } {
    if (!el.size || el.size.width <= 0 || el.size.height <= 0) {
        el.size = defaultSize(kind);
    }
    return el.size;
}

/** Side a port sits on: its fixed `side`, else by direction (in → west,
 * out → east, inout → south, unknown → north). */
function portSide(port: SysmlShapeSchema): 'north' | 'east' | 'south' | 'west' {
    switch (port.side) {
        case 'north':
        case 'east':
        case 'south':
        case 'west':
            return port.side;
    }
    switch (port.direction) {
        case 'in':
            return 'west';
        case 'out':
            return 'east';
        case 'inout':
            return 'south';
        default:
            return 'north';
    }
}

/** Place a shape's unpositioned children, size the shape to fit, then sit its
 * ports on the resulting border. Returns the shape's final size. */
function placeShape(shape: SysmlShapeSchema, direction: string): { width: number; height: number } {
    const kids = childrenOf(shape) as SysmlNodeChildSchema[];
    const blocks = kids.filter(isShapeSchema).filter(c => !isPortSchema(c));
    const ports = kids.filter(isPortSchema);
    const compartments = kids.filter(isCompartmentSchema);
    const labels = kids.filter(isLabelSchema);

    // Nested blocks first (post-order) so a container knows its children's sizes.
    const childSizes = new Map<string, { width: number; height: number }>();
    for (const b of blocks) {
        childSizes.set(b.id, placeShape(b, direction));
    }

    const own = ensureSize(shape, shape.kind);
    let width = own.width;
    let height = own.height;

    if (blocks.length > 0) {
        // Flow unpositioned blocks after whatever is already placed.
        let cursorX = PADDING;
        let cursorY = HEADER;
        let maxX = 0;
        let maxY = 0;
        for (const b of blocks.filter(b => b.position)) {
            const s = childSizes.get(b.id)!;
            maxX = Math.max(maxX, b.position!.x + s.width);
            maxY = Math.max(maxY, b.position!.y + s.height);
        }
        const horizontal = direction === 'RIGHT';
        if (horizontal) {
            cursorX = maxX > 0 ? maxX + GAP : PADDING;
        } else {
            cursorY = maxY > 0 ? maxY + GAP : HEADER;
        }
        for (const b of blocks.filter(b => !b.position)) {
            const s = childSizes.get(b.id)!;
            b.position = { x: cursorX, y: cursorY };
            maxX = Math.max(maxX, cursorX + s.width);
            maxY = Math.max(maxY, cursorY + s.height);
            if (horizontal) {
                cursorX += s.width + GAP;
            } else {
                cursorY += s.height + GAP;
            }
        }
        width = Math.max(width, maxX + PADDING);
        height = Math.max(height, maxY + PADDING);
    }

    // Compartments stack below the header, full width; the block grows to fit.
    let compY = Math.max(own.height, HEADER + 8);
    for (const c of compartments) {
        const h = LINE_HEIGHT * Math.max(1, c.lines.length) + 8;
        if (!c.position) {
            c.position = { x: 0, y: compY };
        }
        if (!c.size) {
            c.size = { width, height: h };
        }
        compY = c.position.y + (c.size?.height ?? h);
        height = Math.max(height, compY);
    }
    for (const c of compartments) {
        if (c.size && c.size.width < width) {
            c.size = { width, height: c.size.height };
        }
    }

    shape.size = { width, height };

    // Ports: evenly spaced along their side, centred on the border.
    const bySide = new Map<string, SysmlShapeSchema[]>();
    for (const p of ports) {
        ensureSize(p, 'port');
        if (p.position) {
            continue;
        }
        const side = portSide(p);
        bySide.set(side, [...(bySide.get(side) ?? []), p]);
    }
    for (const [side, list] of bySide) {
        list.forEach((p, i) => {
            const t = (i + 1) / (list.length + 1);
            const half = PORT_SIZE / 2;
            switch (side) {
                case 'west':
                    p.position = { x: -half, y: height * t - half };
                    break;
                case 'east':
                    p.position = { x: width - half, y: height * t - half };
                    break;
                case 'south':
                    p.position = { x: width * t - half, y: height - half };
                    break;
                default:
                    p.position = { x: width * t - half, y: -half };
            }
        });
    }
    for (const p of ports) {
        placeLabels(p, 'port', PORT_SIZE, PORT_SIZE);
    }

    placeLabels(shape, shape.kind, width, height, labels);
    return shape.size;
}

/** The name label: top-left inside a container, centred in a block, above a port. */
function placeLabels(
    shape: SysmlShapeSchema,
    kind: string,
    width: number,
    height: number,
    labels?: SysmlLabelSchema[],
): void {
    const list = labels ?? (childrenOf(shape) as SysmlNodeChildSchema[]).filter(isLabelSchema);
    for (const l of list) {
        if (l.position) {
            continue;
        }
        if (kind === 'port') {
            l.position = { x: width / 2, y: -3 };
        } else if (isContainerKind(kind)) {
            l.position = { x: 12, y: 18 };
        } else {
            l.position = { x: width / 2, y: height > 40 ? 32 : height / 2 + 4 };
        }
    }
}

export function isContainerKind(kind: string): boolean {
    return kind === 'boundary' || kind === 'system-boundary' || kind === 'swimlane' || kind === 'fragment';
}

/** Give every unpositioned element of `model` a parent-relative position and
 * a size (see the module doc). Mutates the schema in place; safe to re-run. */
export function applyPhase0Layout(model: DiagramModelSchema): void {
    const direction = String(model.layoutOptions?.['elk.direction'] ?? 'DOWN');
    const roots = model.children.filter((c): c is SysmlNodeChildSchema => !isEdgeSchema(c));
    const sizes = new Map<string, { width: number; height: number }>();
    for (const r of roots) {
        if (isShapeSchema(r)) {
            sizes.set(r.id, placeShape(r, direction));
        } else if (isCompartmentSchema(r)) {
            const h = LINE_HEIGHT * Math.max(1, r.lines.length) + 8;
            r.size = r.size ?? { width: 160, height: h };
            sizes.set(r.id, r.size);
        } else {
            sizes.set(r.id, { width: 0, height: 0 });
        }
    }

    // Root grid: rows for a top-down diagram, columns for a left-to-right one,
    // starting after the extent of anything already placed.
    let maxX = 0;
    let maxY = 0;
    for (const r of roots.filter(r => r.position)) {
        const s = sizes.get(r.id)!;
        maxX = Math.max(maxX, r.position!.x + s.width);
        maxY = Math.max(maxY, r.position!.y + s.height);
    }
    const horizontal = direction === 'RIGHT';
    const originX = horizontal && maxX > 0 ? maxX + GAP : GAP;
    const originY = !horizontal && maxY > 0 ? maxY + GAP : GAP;
    const pending = roots.filter(r => !r.position);
    const per = horizontal ? MAX_PER_COLUMN : MAX_PER_ROW;
    let lineStart = 0;
    let cursor = 0;
    let lineExtent = 0;
    pending.forEach((r, i) => {
        const s = sizes.get(r.id)!;
        if (i > 0 && i % per === 0) {
            lineStart += lineExtent + GAP;
            cursor = 0;
            lineExtent = 0;
        }
        if (horizontal) {
            r.position = { x: originX + lineStart, y: originY + cursor };
            cursor += s.height + GAP;
            lineExtent = Math.max(lineExtent, s.width);
        } else {
            r.position = { x: originX + cursor, y: originY + lineStart };
            cursor += s.width + GAP;
            lineExtent = Math.max(lineExtent, s.height);
        }
    });
}
