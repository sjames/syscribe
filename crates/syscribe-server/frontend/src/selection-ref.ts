// Which model element a selection means, for the element side panel
// (`REQ-TRS-VIS-026`). Pure: no DOM, no fetch — `element-panel.ts` shows the
// card and `test/selection-ref.test.mjs` drives this directly.
import { DiagramModelSchema, findShape, isEdgeSchema } from './types';

/** The `ref` to show for the current selection: the single selected shape's or
 * edge's, else `null` (nothing selected, several selected, or the selected
 * thing is not a model element — a label, a compartment, an edge with no ref).
 * Several selected leaves the panel as it is, so the caller must treat `null`
 * as "do not change", not "close". */
export function refForSelection(model: DiagramModelSchema, selectedIds: Iterable<string>): string | null {
    const ids = [...selectedIds];
    if (ids.length !== 1) {
        return null;
    }
    const id = ids[0];
    const shape = findShape(model, id);
    if (shape) {
        return shape.ref && shape.ref !== '' ? shape.ref : null;
    }
    const edge = model.children.filter(isEdgeSchema).find(e => e.id === id);
    return edge?.ref && edge.ref !== '' ? edge.ref : null;
}

/** The URL of the card for `ref`, with `::` as `/` as every other route. */
export function cardUrl(ref: string): string {
    return '/ui/element-card/' + ref.split('::').map(encodeURIComponent).join('/');
}
