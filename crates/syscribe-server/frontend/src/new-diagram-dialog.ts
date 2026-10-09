// DOM glue of the "New diagram" dialog (`REQ-TRS-VIS-023`): populates the
// kind and package lists, fetches subject suggestions for the chosen kind,
// submits through `POST /api/elements` and opens the new diagram in a tab.
// All decisions live in `new-diagram.ts`; this file only reads and writes the
// markup in `templates/index.html`.
import * as api from './api';
import {
    BuildResult,
    buildCreateRequest,
    DIAGRAM_KINDS,
    defaultPackage,
    kindInfo,
    StartFrom,
} from './new-diagram';
import { Finding } from './types';

interface ElementSummary {
    qualifiedName: string;
    elementType: string | null;
}

declare global {
    interface Window {
        NewDiagram: { open(): Promise<void> };
        /** Defined by `base.html`'s tab-management script. */
        openDiagram?: (id: string, name: string, kind: string) => Promise<void>;
    }
}

function el<T extends HTMLElement>(id: string): T {
    const e = document.getElementById(id);
    if (!e) {
        throw new Error(`new-diagram dialog: #${id} missing from the page`);
    }
    return e as T;
}

async function fetchByType(type: string): Promise<ElementSummary[]> {
    const resp = await fetch('/api/elements?type=' + encodeURIComponent(type));
    return resp.ok ? ((await resp.json()) as ElementSummary[]) : [];
}

let candidates: string[] = [];
const packageCache: string[] = [];

async function refreshSubjects(): Promise<void> {
    const kind = el<HTMLSelectElement>('nd-kind').value;
    const info = kindInfo(kind);
    el('nd-subject-hint').textContent = info ? `Subject: ${info.hint}.` : '';
    const lists = await Promise.all((info?.subjectTypes ?? []).map(fetchByType));
    candidates = [...new Set(lists.flat().map(s => s.qualifiedName))].sort();
    const dl = el<HTMLDataListElement>('nd-subjects');
    dl.replaceChildren(
        ...candidates.map(q => {
            const o = document.createElement('option');
            o.value = q;
            return o;
        }),
    );
}

function updateMode(): void {
    // The safety kinds are derived only (GH #223): the blank choice is off for them.
    const deriveOnly = !!kindInfo(el<HTMLSelectElement>('nd-kind').value)?.deriveOnly;
    const radios = Array.from(document.querySelectorAll<HTMLInputElement>('input[name="nd-start"]'));
    for (const r of radios) {
        if (r.value === 'blank') {
            r.disabled = deriveOnly;
        }
        if (deriveOnly && r.value === 'derive') {
            r.checked = true;
        }
    }
    const derive = (document.querySelector('input[name="nd-start"]:checked') as HTMLInputElement).value === 'derive';
    el('nd-subject-label').firstChild!.textContent = derive ? 'Subject ' : 'Subject (optional) ';
}

function showError(msg: string): void {
    const e = el('nd-error');
    e.textContent = msg;
    e.style.display = msg ? 'block' : 'none';
}

function summarize(findings: Finding[]): string {
    return findings.map(f => `${f.code}: ${f.message}`).join('; ');
}

async function open(): Promise<void> {
    const dialog = el<HTMLDialogElement>('new-diagram-dialog');
    showError('');
    const kindSel = el<HTMLSelectElement>('nd-kind');
    if (kindSel.options.length === 0) {
        for (const k of DIAGRAM_KINDS) {
            const o = document.createElement('option');
            o.value = k.kind;
            o.textContent = k.label;
            kindSel.appendChild(o);
        }
    }
    const pkgSel = el<HTMLSelectElement>('nd-package');
    const packages = (await fetchByType('Package')).map(p => p.qualifiedName).filter(q => q !== '').sort();
    packageCache.splice(0, packageCache.length, ...packages);
    pkgSel.replaceChildren(
        ...['', ...packages].map(q => {
            const o = document.createElement('option');
            o.value = q;
            o.textContent = q === '' ? '(model root)' : q;
            return o;
        }),
    );
    pkgSel.value = defaultPackage(packages);
    el<HTMLInputElement>('nd-name').value = '';
    el<HTMLInputElement>('nd-subject').value = '';
    updateMode();
    await refreshSubjects();
    dialog.showModal();
    el<HTMLInputElement>('nd-name').focus();
}

async function submit(ev: Event): Promise<void> {
    ev.preventDefault();
    const startFrom = (document.querySelector('input[name="nd-start"]:checked') as HTMLInputElement).value as StartFrom;
    const built: BuildResult = buildCreateRequest({
        name: el<HTMLInputElement>('nd-name').value,
        kind: el<HTMLSelectElement>('nd-kind').value,
        startFrom,
        subject: el<HTMLInputElement>('nd-subject').value,
        pkg: el<HTMLSelectElement>('nd-package').value,
        candidates,
    });
    if (!built.ok) {
        showError(built.error);
        return;
    }
    const btn = el<HTMLButtonElement>('nd-create');
    btn.disabled = true;
    try {
        const resp = await api.createElement(built.request);
        if (!resp.written) {
            showError(resp.reason ?? (summarize(resp.newErrors) || 'The model refused the new diagram.'));
            return;
        }
        el<HTMLDialogElement>('new-diagram-dialog').close();
        if (window.openDiagram) {
            await window.openDiagram(built.tabId, built.displayName, built.kind);
        }
        if (resp.newWarnings.length > 0) {
            const toast = document.getElementById('sprotty-toast');
            if (toast) {
                toast.textContent = `Created ${built.request.qname}. ${summarize(resp.newWarnings)}`;
                toast.style.display = 'block';
                window.setTimeout(() => (toast.style.display = 'none'), 6000);
            }
        }
    } catch (err) {
        showError(`Could not create the diagram: ${(err as Error).message}`);
    } finally {
        btn.disabled = false;
    }
}

export function installNewDiagramDialog(): void {
    window.NewDiagram = { open };
    document.addEventListener('DOMContentLoaded', () => {
        const form = document.getElementById('nd-form');
        if (!form) {
            return;
        }
        form.addEventListener('submit', ev => void submit(ev));
        el('nd-kind').addEventListener('change', () => {
            updateMode();
            void refreshSubjects();
        });
        document.querySelectorAll('input[name="nd-start"]').forEach(r => r.addEventListener('change', updateMode));
        el('nd-cancel').addEventListener('click', () => el<HTMLDialogElement>('new-diagram-dialog').close());
    });
}
