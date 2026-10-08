// The element side panel of the diagram editor (`REQ-TRS-VIS-026`): shows the
// server-rendered card (`GET /ui/element-card/{qname}`) of the element a
// selected shape or edge depicts. The decision of *which* element is in
// `selection-ref.ts`; this file only moves HTML into the page.
import { cardUrl } from './selection-ref';

declare global {
    interface Window {
        mermaid?: { run(opts: { nodes: Element[] }): Promise<void> };
        htmx?: { process(el: Element): void };
    }
}

let current: string | null = null;
let sequence = 0;

function panel(): HTMLElement | null {
    return document.getElementById('element-panel');
}

/** Forget which element is shown, so the next selection of the same one fetches it again (after a live reload). */
export function invalidateElementCard(): void {
    current = null;
}

/** Show the card of `ref`. A response that arrives after a newer selection is dropped. */
export async function showElementCard(ref: string): Promise<void> {
    const p = panel();
    const body = document.getElementById('element-panel-body');
    if (!p || !body || ref === current) {
        return;
    }
    current = ref;
    const mine = ++sequence;
    p.hidden = false;
    document.getElementById('sprotty-viewport')?.classList.add('has-panel');
    let html: string;
    try {
        const resp = await fetch(cardUrl(ref));
        if (!resp.ok) {
            throw new Error(`HTTP ${resp.status}`);
        }
        html = await resp.text();
    } catch (err) {
        if (mine === sequence) {
            body.textContent = `Could not load ${ref}: ${(err as Error).message}`;
            // Forget the selection so clicking the same shape again retries instead of doing nothing.
            current = null;
        }
        return;
    }
    if (mine !== sequence) {
        return;
    }
    body.innerHTML = html;
    window.htmx?.process(body);
    body.scrollTop = 0;
    const diagrams = Array.from(body.querySelectorAll('pre.mermaid'));
    if (diagrams.length > 0 && window.mermaid) {
        // A diagram that fails to draw must not replace the card that is already shown.
        await window.mermaid.run({ nodes: diagrams }).catch(() => undefined);
    }
}

export function hideElementPanel(): void {
    sequence += 1;
    current = null;
    const p = panel();
    if (p) {
        p.hidden = true;
    }
    document.getElementById('sprotty-viewport')?.classList.remove('has-panel');
}

export function installElementPanel(): void {
    document.addEventListener('DOMContentLoaded', () => {
        document.getElementById('ep-close')?.addEventListener('click', hideElementPanel);
        document.addEventListener('syscribe:reload', invalidateElementCard);
    });
}
