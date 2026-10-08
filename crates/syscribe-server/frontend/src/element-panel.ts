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
    try {
        const resp = await fetch(cardUrl(ref));
        const html = await resp.text();
        if (mine !== sequence) {
            return;
        }
        body.innerHTML = html;
        window.htmx?.process(body);
        const diagrams = Array.from(body.querySelectorAll('pre.mermaid'));
        if (diagrams.length > 0 && window.mermaid) {
            await window.mermaid.run({ nodes: diagrams });
        }
        body.scrollTop = 0;
    } catch (err) {
        if (mine === sequence) {
            body.textContent = `Could not load ${ref}: ${(err as Error).message}`;
        }
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
    });
}
