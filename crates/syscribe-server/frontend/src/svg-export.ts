// *Save companion SVG* (`REQ-TRS-VIS-011`): serialise the live sprotty render
// into a standalone SVG that follows spec §8.16.5 — `xmlns:sysml` on the
// root, `sysml:ref` on every node/port group, `sysml:ref`/`sysml:source`/
// `sysml:target` on every edge group, kind classes kept, a `viewBox` sized to
// the content — with sprotty's interactive state (selection/hover classes,
// generated ids, `tabindex`, the viewport transform) stripped out.
//
// The views put the model facts on the DOM as `data-sysml-*` attributes
// (`views.tsx`); this module only translates them, so the picture the user
// approved and the picture GitHub shows are the same render.

const SVG_NS = 'http://www.w3.org/2000/svg';
const XMLNS_NS = 'http://www.w3.org/2000/xmlns/';
export const SYSML_NS = 'urn:syscribe:1.0';

const INTERACTIVE_CLASSES = ['selected', 'mouseover', 'mousedown', 'mouseleave', 'hover', 'sprotty-graph'];
const PADDING = 12;

/** Serialise the SVG under `hostId`, or `undefined` when nothing is mounted. */
export function serialiseDiagramSvg(hostId: string, diagramQname: string): string | undefined {
    const live = document.querySelector<SVGSVGElement>(`#${hostId} svg`);
    if (!live) {
        return undefined;
    }
    const liveContent = live.querySelector<SVGGElement>(':scope > g');
    if (!liveContent) {
        return undefined;
    }
    // `getBBox()` of the content group is in graph coordinates (it ignores the
    // group's own viewport transform), which is exactly the viewBox we want.
    const bbox = liveContent.getBBox();

    const clone = live.cloneNode(true) as SVGSVGElement;
    const content = clone.querySelector<SVGGElement>(':scope > g');
    content?.removeAttribute('transform');

    clone.setAttributeNS(XMLNS_NS, 'xmlns', SVG_NS);
    clone.setAttributeNS(XMLNS_NS, 'xmlns:sysml', SYSML_NS);
    clone.setAttribute('version', '1.1');
    const width = Math.ceil(bbox.width + 2 * PADDING);
    const height = Math.ceil(bbox.height + 2 * PADDING);
    clone.setAttribute('viewBox', `${Math.floor(bbox.x - PADDING)} ${Math.floor(bbox.y - PADDING)} ${width} ${height}`);
    clone.setAttribute('width', String(width));
    clone.setAttribute('height', String(height));
    clone.setAttribute('data-sysml-diagram', diagramQname);
    clone.removeAttribute('id');
    clone.removeAttribute('tabindex');
    clone.removeAttribute('style');
    clone.classList.remove('sprotty-graph');
    if (clone.getAttribute('class') === '') {
        clone.removeAttribute('class');
    }

    const prefix = `${hostId}_`;
    for (const el of Array.from(clone.querySelectorAll<Element>('*'))) {
        // sprotty ids are `<baseDiv>_<elementId>`; keep the model's own id,
        // which for a manifest diagram is the `shapes:`/`edges:` key.
        const id = el.getAttribute('id');
        if (id && id.startsWith(prefix)) {
            el.setAttribute('id', id.slice(prefix.length));
        }
        for (const cls of INTERACTIVE_CLASSES) {
            el.classList.remove(cls);
        }
        if (el.getAttribute('class') === '') {
            el.removeAttribute('class');
        }
        el.removeAttribute('tabindex');

        const ref = el.getAttribute('data-sysml-ref');
        if (ref !== null) {
            if (ref !== '') {
                el.setAttributeNS(SYSML_NS, 'sysml:ref', ref);
            }
            el.removeAttribute('data-sysml-ref');
        }
        for (const end of ['source', 'target'] as const) {
            const v = el.getAttribute(`data-sysml-${end}`);
            if (v !== null) {
                el.setAttributeNS(SYSML_NS, `sysml:${end}`, v);
                el.removeAttribute(`data-sysml-${end}`);
            }
        }
    }

    // No XML prolog: the server checks that the body starts with `<svg`.
    return new XMLSerializer().serializeToString(clone) + '\n';
}
