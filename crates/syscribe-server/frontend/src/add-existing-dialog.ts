// DOM glue of the "Add existing element" picker (`REQ-TRS-VIS-024`): fetches
// the model's elements once per opening, filters them as the user types, and
// submits through `POST /api/diagrams/shapes/{qname}`. The decisions live in
// `add-existing.ts`; the markup is in `templates/index.html`.
import * as api from './api';
import {
    AddResult,
    buildAddRequest,
    describe,
    ElementSummary,
    searchElements,
} from './add-existing';
import { Finding } from './types';

export interface AddExistingOptions {
    diagramQname: string;
    /** Called after the server accepted the shape, before the dialog closes. */
    onAdded: () => Promise<void>;
}

function el<T extends HTMLElement>(id: string): T {
    const e = document.getElementById(id);
    if (!e) {
        throw new Error(`add-existing dialog: #${id} missing from the page`);
    }
    return e as T;
}

function showError(msg: string): void {
    const e = el('ae-error');
    e.textContent = msg;
    e.style.display = msg ? 'block' : 'none';
}

function summarize(findings: Finding[]): string {
    return findings.map(f => `${f.code}: ${f.message}`).join('; ');
}

let all: ElementSummary[] = [];
let current: AddExistingOptions | null = null;

function refreshResults(): void {
    const query = el<HTMLInputElement>('ae-ref').value;
    const hits = searchElements(all, query);
    el<HTMLDataListElement>('ae-results').replaceChildren(
        ...hits.map(e => {
            const o = document.createElement('option');
            o.value = e.qualifiedName;
            o.label = `${describe(e)} — ${e.qualifiedName}`;
            return o;
        }),
    );
    el('ae-count').textContent = hits.length === 0 ? 'No matches.' : `${hits.length}${hits.length >= 100 ? '+' : ''} match${hits.length === 1 ? '' : 'es'}`;
}

export async function openAddExisting(opts: AddExistingOptions): Promise<void> {
    current = opts;
    showError('');
    const resp = await fetch('/api/elements');
    all = resp.ok ? ((await resp.json()) as ElementSummary[]) : [];
    el<HTMLInputElement>('ae-ref').value = '';
    refreshResults();
    const dialog = el<HTMLDialogElement>('add-existing-dialog');
    dialog.showModal();
    el<HTMLInputElement>('ae-ref').focus();
}

async function submit(ev: Event): Promise<void> {
    ev.preventDefault();
    if (!current) {
        return;
    }
    const built: AddResult = buildAddRequest({ ref: el<HTMLInputElement>('ae-ref').value, all });
    if (!built.ok) {
        showError(built.error);
        return;
    }
    const btn = el<HTMLButtonElement>('ae-add');
    btn.disabled = true;
    try {
        const resp = await api.addShape(current.diagramQname, built.request);
        if (!resp.written) {
            showError(resp.reason ?? (summarize(resp.newErrors) || 'The model refused the shape.'));
            return;
        }
        el<HTMLDialogElement>('add-existing-dialog').close();
        await current.onAdded();
    } catch (err) {
        showError(`Could not add the element: ${(err as Error).message}`);
    } finally {
        btn.disabled = false;
    }
}

export function installAddExistingDialog(): void {
    document.addEventListener('DOMContentLoaded', () => {
        const form = document.getElementById('ae-form');
        if (!form) {
            return;
        }
        form.addEventListener('submit', ev => void submit(ev));
        el('ae-ref').addEventListener('input', refreshResults);
        el('ae-cancel').addEventListener('click', () => el<HTMLDialogElement>('add-existing-dialog').close());
    });
}
