// The feature model page (`/features`, `REQ-TRS-FMED-001`, `-002`): loads the
// feature diagram and the SAT analysis from the server, draws the diagram with
// the shared sprotty container, and drives collapse, search, the analysis
// overlay, the Inspector and live refresh. The decisions that need no DOM live
// in `feature-core.ts`.
import 'reflect-metadata';
import { IActionDispatcher, LocalModelSource, TYPES } from 'sprotty';
import { CenterAction, FitToScreenAction } from 'sprotty-protocol';
import { createDiagramContainer } from './container';
import * as api from './api';
import {
    applyAnalysis,
    applyConfiguration,
    bannerText,
    cellGlyph,
    compareConfigs,
    configMatrix,
    collapseBelow,
    configCounts,
    ConfigureResult,
    configurationFields,
    configurationPackage,
    buildParameter,
    deltaLines,
    describeConflict,
    descendantIds,
    dropTarget,
    EditHistory,
    EditOp,
    EditResult,
    expandOneLevel,
    FeatureAnalysis,
    featureNodes,
    formOf,
    ImpactResult,
    impactSummary,
    nextChoice,
    parameterSummary,
    productsText,
    revealing,
    rootIds,
    search,
    shapeId,
    StoredConfiguration,
    summaryLines,
    viewMode,
    visibleModel,
    withChoice,
} from './feature-core';
import { joinQname, validateName } from './new-diagram';
import { prepareForLayout } from './layout';
import { DiagramModelSchema, isEdgeSchema, SysmlNodeSchema } from './types';

const HOST = 'fm-host';

function byId<T extends HTMLElement>(id: string): T {
    const e = document.getElementById(id);
    if (!e) {
        throw new Error(`feature page: #${id} missing`);
    }
    return e as T;
}

function esc(s: string): string {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

/** The laid-out box of every feature node of `view` (root-relative: feature nodes are root children). */
function layoutBoxes(view: DiagramModelSchema): Map<string, { x: number; y: number; w: number; h: number }> {
    const out = new Map<string, { x: number; y: number; w: number; h: number }>();
    for (const n of featureNodes(view)) {
        const at = (n as unknown as { position?: { x: number; y: number } }).position;
        const size = (n as unknown as { size?: { width: number; height: number } }).size;
        if (at && size) {
            out.set(n.id, { x: at.x, y: at.y, w: size.width, h: size.height });
        }
    }
    return out;
}

class FeaturePage {
    private full: DiagramModelSchema | null = null;
    private analysis: FeatureAnalysis | null = null;
    private collapsed = new Set<string>();
    private selected: string | null = null;
    private matches: SysmlNodeSchema[] = [];
    private matchIndex = 0;
    private busy = false;
    private again = false;
    private firstLoad = true;
    private configMode = false;
    private choices: Record<string, boolean> = {};
    private config: ConfigureResult | null = null;
    private stored: StoredConfiguration[] = [];
    private message = '';
    private loaded = '';
    private downAt: { x: number; y: number } | null = null;
    private editMode = false;
    private matrixMode = false;
    private compare: [number, number] = [0, 1];
    private readonly history = new EditHistory();
    private boxes = new Map<string, { x: number; y: number; w: number; h: number }>();
    private toastTimer = 0;
    private readonly dispatcher: IActionDispatcher;
    private readonly source: LocalModelSource;

    constructor() {
        const container = createDiagramContainer(HOST, {
            // The diagram is a view of the model; a dragged feature snaps back.
            onMoveFinished: moves => void this.onMoved(moves.map(m => ({ id: m.elementId, x: m.toPosition.x, y: m.toPosition.y }))),
            onSelectionChanged: (sel, desel) => this.onSelection(sel, desel),
        });
        this.dispatcher = container.get<IActionDispatcher>(TYPES.IActionDispatcher);
        this.source = container.get(LocalModelSource);

        byId('fm-collapse').addEventListener('click', () => {
            if (this.full) {
                this.collapsed = collapseBelow(this.full, 1);
                void this.render(true);
            }
        });
        byId('fm-expand').addEventListener('click', () => {
            this.collapsed.clear();
            void this.render(true);
        });
        byId('fm-fit').addEventListener('click', () => void this.fit());
        byId('fm-expand-level').addEventListener('click', () => {
            if (this.full) {
                this.collapsed = expandOneLevel(this.full, this.collapsed);
                void this.render(true);
            }
        });
        byId('fm-configure').addEventListener('click', () => void this.setConfigMode(!this.configMode));
        byId('fm-edit').addEventListener('click', () => void this.setEditMode(!this.editMode));
        byId('fm-matrix').addEventListener('click', () => this.setMatrixMode(!this.matrixMode));
        byId('fm-matrix-view').addEventListener('click', ev => this.onMatrixClick(ev));
        byId('fm-matrix-view').addEventListener('change', ev => this.onMatrixChange(ev));
        byId('fm-undo').addEventListener('click', () => void this.undo());
        byId('fm-redo').addEventListener('click', () => void this.redo());
        byId('fm-selected').addEventListener('click', ev => {
            const rb = (ev.target as Element).closest('button[data-action="removeBinding"]') as HTMLButtonElement | null;
            if (rb && this.selectedNode()) {
                void this.runEdit({ op: 'removeBinding', configuration: rb.dataset.config ?? '', feature: this.selectedNode()!.ref, name: rb.dataset.name ?? '' });
                return;
            }
            const b = (ev.target as Element).closest('button[data-impact="removal"]') as HTMLButtonElement | null;
            if (b) {
                void this.previewRemoval(b.dataset.feature ?? '', b.dataset.subtree === 'true');
            }
        });
        byId('fm-edit-panel').addEventListener('click', ev => this.onEditPanel(ev));
        byId('fm-edit-panel').addEventListener('change', ev => this.onEditChange(ev));
        document.addEventListener('keydown', ev => this.onKey(ev));
        const canvas = byId('fm-canvas');
        canvas.addEventListener('mousedown', ev => (this.downAt = { x: ev.clientX, y: ev.clientY }));
        canvas.addEventListener('click', ev => this.onCanvasClick(ev));
        byId('fm-config').addEventListener('click', ev => this.onConfigPanel(ev));
        byId('fm-config').addEventListener('change', ev => this.onConfigChange(ev));
        const box = byId<HTMLInputElement>('fm-search');
        box.addEventListener('input', () => this.onSearch(box.value));
        box.addEventListener('keydown', ev => {
            if (ev.key === 'Enter') {
                this.nextMatch(ev.shiftKey ? -1 : 1);
            }
        });
        // On the stable parent: sprotty replaces the host element when it first patches the DOM.
        byId('fm-canvas').addEventListener('click', ev => {
            const t = (ev.target as Element).closest('[data-fm-toggle]');
            if (t) {
                ev.stopPropagation();
                this.toggle(t.getAttribute('data-fm-toggle') ?? '');
            }
        });
        document.addEventListener('syscribe:reload', () => void this.load());
        window.addEventListener('syscribe:socket', e => {
            const badge = byId('fm-live');
            const open = (e as CustomEvent<{ open: boolean }>).detail.open;
            badge.classList.toggle('off', !open);
            badge.textContent = open ? 'live' : 'offline';
        });
        void this.load();
    }

    /** Fetch the diagram and the analysis, then draw. Overlapping calls coalesce. */
    async load(): Promise<void> {
        if (this.busy) {
            this.again = true;
            return;
        }
        this.busy = true;
        try {
            do {
                this.again = false;
                const [diagram, analysis] = await Promise.all([
                    fetch('/api/feature-model/diagram').then(r => r.json() as Promise<DiagramModelSchema>),
                    fetch('/api/feature-model/analysis').then(r => r.json() as Promise<FeatureAnalysis>),
                ]);
                this.full = diagram;
                this.analysis = analysis;
                this.stored = await fetch('/api/feature-model/configurations').then(r => r.json() as Promise<StoredConfiguration[]>);
                if (this.configMode) {
                    this.config = await this.configure(this.choices);
                }
                if (this.firstLoad) {
                    this.firstLoad = false;
                    // A large model opens collapsed to its first two levels.
                    if (featureNodes(diagram).length > 60) {
                        this.collapsed = collapseBelow(diagram, 3);
                    }
                }
                await this.render();
            } while (this.again);
        } catch (err) {
            byId('fm-banner').hidden = false;
            byId('fm-banner').textContent = `Could not load the feature model: ${(err as Error).message}`;
        } finally {
            this.busy = false;
        }
    }

    private async render(fit = false): Promise<void> {
        if (!this.full) {
            return;
        }
        const empty = featureNodes(this.full).length === 0;
        byId('fm-empty').hidden = !empty;
        const banner = bannerText(this.analysis);
        byId('fm-banner').hidden = banner === null;
        byId('fm-banner').textContent = banner ?? '';
        byId('fm-summary').innerHTML = summaryLines(this.analysis).map(l => `<div>${esc(l)}</div>`).join('');
        if (empty) {
            return;
        }
        // A fresh copy each time: layout writes sizes and positions back into the schema.
        const copy = JSON.parse(JSON.stringify(this.full)) as DiagramModelSchema;
        applyAnalysis(copy, this.analysis);
        applyConfiguration(copy, this.configMode ? this.config : null);
        const matched = new Set(this.matches.map(m => m.id));
        for (const n of featureNodes(copy)) {
            n.matched = matched.has(n.id);
        }
        const view = visibleModel(copy, this.collapsed);
        prepareForLayout(view);
        await this.source.setModel(view);
        this.boxes = layoutBoxes(view);
        if (fit || this.firstRender) {
            this.firstRender = false;
            await this.fit();
        }
        this.showSelected();
        this.renderEditPanel();
        if (this.matrixMode) {
            this.renderMatrix();
        }
    }
    private firstRender = true;

    // -----------------------------------------------------------------
    // The configurator (REQ-TRS-FMED-003)
    // -----------------------------------------------------------------

    private async configure(selection: Record<string, boolean>): Promise<ConfigureResult> {
        const resp = await fetch('/api/feature-model/configure', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ selection }),
        });
        return (await resp.json()) as ConfigureResult;
    }

    private nameOf(qname: string): string {
        return (this.full && featureNodes(this.full).find(n => n.ref === qname)?.name) || qname.split('::').pop() || qname;
    }

    private async setConfigMode(on: boolean): Promise<void> {
        this.configMode = on;
        const btn = byId('fm-configure');
        btn.classList.toggle('active', on);
        btn.setAttribute('aria-pressed', String(on));
        byId('fm-config').hidden = !on;
        this.message = '';
        this.config = on ? await this.configure(this.choices) : null;
        this.renderConfigPanel();
        await this.render();
    }

    /** A click on a feature in configure mode cycles its choice; a drag is not a click. */
    private onCanvasClick(ev: MouseEvent): void {
        const down = this.downAt;
        this.downAt = null;
        if (!this.configMode || (ev.target as Element).closest('[data-fm-toggle]')) {
            return;
        }
        if (down && Math.hypot(ev.clientX - down.x, ev.clientY - down.y) > 4) {
            return;
        }
        const g = (ev.target as Element).closest('g.sysml-node.feature');
        const ref = g?.getAttribute('data-sysml-ref');
        if (!ref) {
            return;
        }
        void this.choose(ref, nextChoice(this.choices[ref]));
    }

    /** Apply a choice if some product still satisfies it; otherwise keep the old
     * choices and say which choices clash and why. */
    private async choose(feature: string, value: boolean | undefined): Promise<void> {
        const next = withChoice(this.choices, feature, value);
        const result = await this.configure(next);
        if (!result.satisfiable) {
            this.message = describeConflict(result, q => this.nameOf(q));
            this.renderConfigPanel();
            return;
        }
        this.choices = next;
        this.config = result;
        this.message = '';
        this.renderConfigPanel();
        await this.render();
    }

    private async replaceChoices(next: Record<string, boolean>): Promise<void> {
        const result = await this.configure(next);
        this.choices = next;
        this.config = result;
        this.message = result.satisfiable ? '' : describeConflict(result, q => this.nameOf(q)) + ' The loaded configuration is not a valid product.';
        this.renderConfigPanel();
        await this.render();
    }

    private renderConfigPanel(): void {
        const pane = byId('fm-config');
        if (!this.configMode) {
            pane.innerHTML = '';
            return;
        }
        const r = this.config;
        const c = configCounts(r);
        const options = ['<option value="">(start from nothing)</option>']
            .concat(this.stored.map(s => `<option value="${esc(s.qname)}"${s.qname === this.loaded ? ' selected' : ''}>${esc(s.name)}${s.id ? ` (${esc(s.id)})` : ''}</option>`))
            .join('');
        const status = r && !r.satisfiable ? 'invalid' : productsText(r);
        pane.innerHTML = `
          <div class="fm-config-head">Configuration</div>
          <label class="fm-config-row">Start from <select id="fm-conf-load">${options}</select></label>
          <div class="fm-config-count" id="fm-conf-count">${esc(status)}</div>
          <div class="fm-config-sub">${c.chosen} chosen, ${c.implied} implied, ${c.open} open</div>
          <div class="fm-config-msg" id="fm-conf-msg"${this.message ? '' : ' hidden'}>${esc(this.message)}</div>
          <div class="fm-config-row">
            <button class="canvas-btn" id="fm-conf-clear">Clear</button>
          </div>
          <div class="fm-config-row">
            <input id="fm-conf-name" type="text" placeholder="Name for a new Configuration" autocomplete="off">
            <button class="canvas-btn" id="fm-conf-save"${r && r.satisfiable ? '' : ' disabled'}>Save</button>
          </div>
          <div class="fm-config-hint">Click a feature to select it, again to deselect it, again to leave it open. Features the model forces are shown as rings.</div>`;
    }

    private onConfigChange(ev: Event): void {
        const t = ev.target as HTMLSelectElement;
        if (t.id !== 'fm-conf-load') {
            return;
        }
        this.loaded = t.value;
        const conf = this.stored.find(s => s.qname === t.value);
        void this.replaceChoices(conf ? { ...conf.selection } : {});
    }

    private onConfigPanel(ev: MouseEvent): void {
        const id = (ev.target as Element).closest('button')?.id;
        if (id === 'fm-conf-clear') {
            this.loaded = '';
            void this.replaceChoices({});
        } else if (id === 'fm-conf-save') {
            void this.saveConfiguration();
        }
    }

    /** Write the product the server completed from the choices as a new `Configuration`. */
    private async saveConfiguration(): Promise<void> {
        const r = this.config;
        const input = byId<HTMLInputElement>('fm-conf-name');
        if (!r || !r.satisfiable) {
            return;
        }
        const problem = validateName(input.value);
        if (problem) {
            this.message = problem;
            this.renderConfigPanel();
            return;
        }
        const name = input.value.trim();
        const qname = joinQname(configurationPackage(this.stored), name);
        const resp = await api.createElement({ qname, type: 'Configuration', fields: configurationFields(r, name) });
        this.message = resp.written
            ? `Saved ${qname}.`
            : (resp.reason ?? resp.newErrors.map(f => `${f.code}: ${f.message}`).join('; ')) || 'The model refused the configuration.';
        this.renderConfigPanel();
    }

    // -----------------------------------------------------------------
    // Editing (REQ-TRS-FMED-004)
    // -----------------------------------------------------------------

    private async setEditMode(on: boolean): Promise<void> {
        if (on && this.configMode) {
            await this.setConfigMode(false);
        }
        this.editMode = on;
        const btn = byId('fm-edit');
        btn.classList.toggle('active', on);
        btn.setAttribute('aria-pressed', String(on));
        byId('fm-edit-panel').hidden = !on;
        this.updateHistoryButtons();
        this.renderEditPanel();
    }

    private updateHistoryButtons(): void {
        const undo = byId<HTMLButtonElement>('fm-undo');
        const redo = byId<HTMLButtonElement>('fm-redo');
        undo.hidden = redo.hidden = !this.editMode;
        undo.disabled = !this.history.canUndo;
        redo.disabled = !this.history.canRedo;
    }

    private toast(text: string, error = false): void {
        const el = byId('fm-toast');
        el.textContent = text;
        el.classList.toggle('error', error);
        el.hidden = false;
        window.clearTimeout(this.toastTimer);
        this.toastTimer = window.setTimeout(() => (el.hidden = true), error ? 7000 : 4500);
    }

    /** Ask before an edit that makes things worse, or removes a subtree. */
    private confirm(title: string, lines: string[], ok: string): Promise<boolean> {
        const dlg = byId<HTMLDialogElement>('fm-confirm');
        byId('fm-confirm-title').textContent = title;
        byId('fm-confirm-lines').innerHTML = lines.map(l => `<li>${esc(l)}</li>`).join('');
        byId('fm-confirm-ok').textContent = ok;
        return new Promise(resolve => {
            dlg.addEventListener('close', () => resolve(dlg.returnValue === 'ok'), { once: true });
            dlg.returnValue = 'cancel';
            dlg.showModal();
        });
    }

    private async post(op: EditOp, acceptWorse: boolean): Promise<EditResult> {
        const resp = await fetch('/api/feature-model/edit', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ edit: op, preview: false, acceptWorse }),
        });
        return (await resp.json()) as EditResult;
    }

    /** Run one edit. An edit that makes the model worse is held and shown first;
     * the user confirms, or nothing is written. Returns the result when it was written. */
    private async runEdit(op: EditOp, opts: { record?: boolean; acceptWorse?: boolean } = {}): Promise<EditResult | null> {
        let res = await this.post(op, opts.acceptWorse ?? false);
        if (res.needsConfirmation && res.delta) {
            const go = await this.confirm('This edit makes the feature model worse', deltaLines(res.delta, q => this.nameOf(q), true), 'Apply anyway');
            if (!go) {
                await this.render();
                return null;
            }
            res = await this.post(op, true);
        }
        if (!res.written) {
            this.toast(res.reason ?? 'The edit was refused.', true);
            await this.render();
            return null;
        }
        if (opts.record !== false && res.undo) {
            this.history.record(res.undo);
        }
        if (res.delta) {
            const notes = deltaLines(res.delta, q => this.nameOf(q));
            if (notes.length) {
                this.toast(notes.join(' '));
            }
        }
        if (res.feature) {
            this.selected = shapeId(res.feature);
        }
        this.updateHistoryButtons();
        await this.load();
        return res;
    }

    private async undo(): Promise<void> {
        const op = this.history.nextUndo();
        if (!op) {
            return;
        }
        const res = await this.runEdit(op, { record: false, acceptWorse: true });
        if (res?.undo) {
            this.history.undone(res.undo);
        }
        this.updateHistoryButtons();
    }

    private async redo(): Promise<void> {
        const op = this.history.nextRedo();
        if (!op) {
            return;
        }
        const res = await this.runEdit(op, { record: false, acceptWorse: true });
        if (res?.undo) {
            this.history.redone(res.undo);
        }
        this.updateHistoryButtons();
    }

    private onKey(ev: KeyboardEvent): void {
        if (!this.editMode || !(ev.ctrlKey || ev.metaKey) || (ev.target as Element).closest('input, textarea, select')) {
            return;
        }
        if (ev.key.toLowerCase() === 'z') {
            ev.preventDefault();
            void (ev.shiftKey ? this.redo() : this.undo());
        } else if (ev.key.toLowerCase() === 'y') {
            ev.preventDefault();
            void this.redo();
        }
    }

    /** A feature dragged in edit mode and dropped on another becomes its child; anywhere else it snaps back. */
    private async onMoved(moves: { id: string; x: number; y: number }[]): Promise<void> {
        const m = moves[0];
        const node = this.full ? featureNodes(this.full).find(n => n.id === m?.id) : undefined;
        const box = m ? this.boxes.get(m.id) : undefined;
        if (!this.editMode || !m || !node || !box || !this.full) {
            await this.render();
            return;
        }
        const target = dropTarget(this.boxes, m.id, { x: m.x + box.w / 2, y: m.y + box.h / 2 }, descendantIds(this.full, m.id));
        const parent = target ? featureNodes(this.full).find(n => n.id === target) : undefined;
        const current = [...this.full.children].filter(isEdgeSchema).find(e => e.kind === 'child' && e.targetId === m.id)?.sourceId;
        if (!parent || parent.id === current) {
            await this.render();
            return;
        }
        await this.runEdit({ op: 'move', feature: node.ref, newParent: parent.ref });
    }

    private selectedNode(): SysmlNodeSchema | undefined {
        return this.full && this.selected ? featureNodes(this.full).find(n => n.id === this.selected) : undefined;
    }

    private renderEditPanel(): void {
        const pane = byId('fm-edit-panel');
        if (!this.editMode || !this.full) {
            pane.innerHTML = '';
            return;
        }
        const all = featureNodes(this.full);
        const node = this.selectedNode();
        const opt = (n: SysmlNodeSchema): string => `<option value="${esc(n.ref)}">${esc(n.name)} (${esc(n.ref)})</option>`;
        if (!node) {
            pane.innerHTML = `
              <div class="fm-edit-head">Edit</div>
              <div class="fm-edit-row"><label>Add a root feature</label></div>
              <div class="fm-edit-row"><input id="fm-e-root-name" type="text" placeholder="Name" autocomplete="off"><button data-action="addRoot">Add</button></div>
              <div class="fm-edit-hint">Select a feature to change it. Drag a feature onto another to make it that feature's child.</div>`;
            return;
        }
        const m = node.feature;
        const below = descendantIds(this.full, node.id);
        const others = all.filter(n => n.id !== node.id && !below.has(n.id));
        const constraint = (kind: 'requires' | 'excludes', target: string): string =>
            `<li>${kind} <b>${esc(this.nameOf(target))}</b><button data-action="removeConstraint" data-kind="${kind}" data-target="${esc(target)}" title="Remove this constraint">&#x2715;</button></li>`;
        const params = (m?.parameters ?? [])
            .map(
                d =>
                    `<li><span>${esc(parameterSummary(d))}</span><button data-action="editParameter" data-name="${esc(String(d.name))}" title="Edit this parameter">&#x270E;</button><button data-action="removeParameter" data-name="${esc(String(d.name))}" title="Remove this parameter and its bindings">&#x2715;</button></li>`,
            )
            .join('');
        const declared = [...(m?.requires ?? []).map(t => constraint('requires', t)), ...(m?.excludes ?? []).map(t => constraint('excludes', t))].join('');
        pane.innerHTML = `
          <div class="fm-edit-head">Edit ${esc(node.name)}</div>
          <div class="fm-edit-row"><input id="fm-e-name" type="text" value="${esc(node.name)}" autocomplete="off"><button data-action="rename">Rename</button></div>
          <div class="fm-edit-row">
            <label>Membership</label>
            <select id="fm-e-mandatory"><option value="false"${m?.mandatory ? '' : ' selected'}>optional</option><option value="true"${m?.mandatory ? ' selected' : ''}>mandatory</option></select>
            <label>Children are</label>
            <select id="fm-e-group">${['optional', 'alternative', 'or'].map(g => `<option value="${g}"${m?.group === g ? ' selected' : ''}>${g === 'optional' ? 'free' : g === 'alternative' ? 'XOR' : 'OR'}</option>`).join('')}</select>
          </div>
          <div class="fm-edit-row"><label title="A grouping feature with no implementation of its own; it is still selectable"><input id="fm-e-abstract" type="checkbox"${node.isAbstract ? ' checked' : ''}> abstract</label></div>
          <div class="fm-edit-row"><input id="fm-e-child" type="text" placeholder="New child feature" autocomplete="off"><button data-action="addChild">Add child</button></div>
          <div class="fm-edit-row"><label>Constraints</label></div>
          <ul class="fm-edit-constraints">${declared || '<li class="detail-empty">none declared</li>'}</ul>
          <div class="fm-edit-row">
            <select id="fm-e-ckind"><option value="requires">requires</option><option value="excludes">excludes</option></select>
            <select id="fm-e-ctarget">${others.map(opt).join('')}</select>
            <button data-action="addConstraint">Add</button>
          </div>
          <div class="fm-edit-row"><label>Parameters</label></div>
          <ul class="fm-edit-constraints">${params || '<li class="detail-empty">none declared</li>'}</ul>
          <div class="fm-edit-row">
            <input id="fm-p-name" type="text" placeholder="name" size="8" autocomplete="off">
            <input id="fm-p-type" type="text" placeholder="type (ScalarValues::Real)" autocomplete="off">
          </div>
          <div class="fm-edit-row">
            <input id="fm-p-range" type="text" placeholder="range 0..=10" size="8" autocomplete="off">
            <input id="fm-p-default" type="text" placeholder="default" size="6" autocomplete="off">
            <label><input id="fm-p-required" type="checkbox"> required</label>
            <button data-action="setParameter">Set</button>
          </div>
          <div class="fm-edit-row">
            <label>Move under</label>
            <select id="fm-e-parent"><option value="">(a root)</option>${others.map(opt).join('')}</select>
            <button data-action="move">Move</button>
          </div>
          <div class="fm-edit-row"><button class="fm-danger" data-action="remove">${below.size ? `Remove with ${below.size} below` : 'Remove'}</button></div>
          <div class="fm-edit-hint">Every change is checked first; one that makes a feature dead or the model void asks before it is written. Ctrl+Z undoes.</div>`;
    }

    private onEditPanel(ev: MouseEvent): void {
        const btn = (ev.target as Element).closest('button[data-action]') as HTMLButtonElement | null;
        if (!btn) {
            return;
        }
        const val = (id: string): string => (document.getElementById(id) as HTMLInputElement | HTMLSelectElement | null)?.value ?? '';
        const node = this.selectedNode();
        const feature = node?.ref ?? '';
        switch (btn.dataset.action) {
            case 'addRoot':
                void this.runEdit({ op: 'add', name: val('fm-e-root-name') });
                break;
            case 'rename':
                void this.runEdit({ op: 'rename', feature, name: val('fm-e-name') });
                break;
            case 'addChild':
                void this.runEdit({ op: 'add', parent: feature, name: val('fm-e-child') });
                break;
            case 'addConstraint':
                void this.runEdit({ op: 'addConstraint', feature, kind: val('fm-e-ckind'), target: val('fm-e-ctarget') });
                break;
            case 'editParameter': {
                const f = formOf((node?.feature?.parameters ?? []).find(d => d.name === btn.dataset.name));
                const put = (id: string, v: string): void => {
                    (document.getElementById(id) as HTMLInputElement).value = v;
                };
                put('fm-p-name', f.name);
                put('fm-p-type', f.type);
                put('fm-p-range', f.range);
                put('fm-p-default', f.defaultValue);
                (document.getElementById('fm-p-required') as HTMLInputElement).checked = f.required;
                break;
            }
            case 'setParameter': {
                const name = val('fm-p-name').trim();
                const existing = (node?.feature?.parameters ?? []).find(d => d.name === name);
                const parameter = buildParameter(existing, {
                    name,
                    type: val('fm-p-type'),
                    range: val('fm-p-range'),
                    defaultValue: val('fm-p-default'),
                    required: (document.getElementById('fm-p-required') as HTMLInputElement).checked,
                });
                void this.runEdit({ op: 'setParameter', feature, parameter });
                break;
            }
            case 'removeParameter':
                void this.runEdit({ op: 'removeParameter', feature, name: btn.dataset.name ?? '' });
                break;
            case 'removeConstraint':
                void this.runEdit({ op: 'removeConstraint', feature, kind: btn.dataset.kind ?? '', target: btn.dataset.target ?? '' });
                break;
            case 'move':
                void this.runEdit({ op: 'move', feature, ...(val('fm-e-parent') ? { newParent: val('fm-e-parent') } : {}) });
                break;
            case 'remove': {
                const below = node && this.full ? descendantIds(this.full, node.id).size : 0;
                void this.confirm(`Remove ${node?.name ?? 'the feature'}?`, [below ? `This also removes the ${below} feature${below === 1 ? '' : 's'} below it.` : 'The feature is deleted.', 'Constraints and configuration choices that name it are removed with it.'], 'Remove').then(ok => {
                    if (ok) {
                        void this.runEdit({ op: 'remove', feature, subtree: below > 0 });
                    }
                });
                break;
            }
        }
    }

    /** The rename and field inputs change on their own, without a button. */
    private onEditChange(ev: Event): void {
        const t = ev.target as HTMLSelectElement;
        const node = this.selectedNode();
        if (!node) {
            return;
        }
        if (t.id === 'fm-e-abstract') {
            void this.runEdit({ op: 'setAbstract', feature: node.ref, isAbstract: (t as unknown as HTMLInputElement).checked });
        } else if (t.id === 'fm-e-mandatory') {
            void this.runEdit({ op: 'setMandatory', feature: node.ref, mandatory: t.value === 'true' });
        } else if (t.id === 'fm-e-group') {
            void this.runEdit({ op: 'setGroup', feature: node.ref, groupKind: t.value });
        }
    }

    // -----------------------------------------------------------------
    // Matrix and comparison (REQ-TRS-FMED-006)
    // -----------------------------------------------------------------

    private setMatrixMode(on: boolean): void {
        this.matrixMode = on;
        const btn = byId('fm-matrix');
        btn.classList.toggle('active', on);
        btn.setAttribute('aria-pressed', String(on));
        byId('fm-matrix-view').hidden = !on;
        if (on) {
            this.renderMatrix();
        }
    }

    private renderMatrix(): void {
        const pane = byId('fm-matrix-view');
        if (!this.full) {
            return;
        }
        if (this.stored.length === 0) {
            pane.innerHTML = '<p class="detail-empty">This model has no Configurations to compare. Use Configure to make one.</p>';
            return;
        }
        const query = byId<HTMLInputElement>('fm-search').value;
        const [ia, ib] = [Math.min(this.compare[0], this.stored.length - 1), Math.min(this.compare[1], this.stored.length - 1)];
        const rows = configMatrix(this.full, this.collapsed, this.stored, query, [ia, ib]);
        const opts = (sel: number): string => this.stored.map((s, i) => `<option value="${i}"${i === sel ? ' selected' : ''}>${esc(s.id ?? s.name)}</option>`).join('');
        const diff = compareConfigs(this.full, this.stored[ia].selection, this.stored[ib].selection);
        const list = (qs: string[]): string => (qs.length ? qs.map(q => esc(this.nameOf(q))).join(', ') : 'nothing');
        const head = this.stored.map(s => `<th title="${esc(s.name)}">${esc(s.id ?? s.name)}</th>`).join('');
        const body = rows
            .map(r => {
                const cells = r.cells.map(c => `<td class="fm-mx-cell ${c === true ? 'fm-mx-on' : c === false ? 'fm-mx-off' : ''}">${cellGlyph(c)}</td>`).join('');
                return `<tr class="${r.differs ? 'fm-mx-differs' : ''}${r.id === this.selected ? ' fm-mx-sel' : ''}" data-id="${esc(r.id)}"><td class="fm-mx-name" style="padding-left:${8 + r.depth * 14}px">${esc(r.name)}</td>${cells}</tr>`;
            })
            .join('');
        pane.innerHTML = `
          <div class="fm-mx-controls">
            <b>Compare</b>
            <select id="fm-mx-a">${opts(ia)}</select> with <select id="fm-mx-b">${opts(ib)}</select>
            <span>${rows.length} of ${featureNodes(this.full).length} features shown; a row is highlighted where the two differ</span>
          </div>
          <div class="fm-mx-diff">
            <div><b>Only in ${esc(this.stored[ia].id ?? this.stored[ia].name)}:</b> ${list(diff.onlyA)}</div>
            <div><b>Only in ${esc(this.stored[ib].id ?? this.stored[ib].name)}:</b> ${list(diff.onlyB)}</div>
            <div>${diff.both} features in both, ${diff.neither} in neither.</div>
          </div>
          <table class="fm-mx"><thead><tr><th class="fm-mx-name">Feature</th>${head}</tr></thead><tbody>${body}</tbody></table>`;
    }

    private onMatrixClick(ev: MouseEvent): void {
        const row = (ev.target as Element).closest('tr[data-id]');
        if (row) {
            this.selected = row.getAttribute('data-id');
            this.showSelected();
            this.renderEditPanel();
            this.renderMatrix();
        }
    }

    private onMatrixChange(ev: Event): void {
        const t = ev.target as HTMLSelectElement;
        if (t.id === 'fm-mx-a') {
            this.compare = [Number(t.value), this.compare[1]];
        } else if (t.id === 'fm-mx-b') {
            this.compare = [this.compare[0], Number(t.value)];
        } else {
            return;
        }
        this.renderMatrix();
    }

    // -----------------------------------------------------------------
    // Impact (REQ-TRS-FMED-005)
    // -----------------------------------------------------------------

    private renderImpact(i: ImpactResult, node: SysmlNodeSchema): string {
        const link = (e: { qname: string; name: string }): string =>
            `<li><a href="#" hx-get="/ui/detail/${e.qname.split('::').map(encodeURIComponent).join('/')}" hx-target="#modal-content" hx-swap="innerHTML">${esc(e.name)}</a></li>`;
        const lines = impactSummary(i).map(l => `<div class="fm-impact-line">${esc(l)}</div>`).join('');
        const gated = (i.gates?.byType ?? [])
            .map(t => `<details><summary>${t.count} ${esc(t.type)}</summary><ul>${t.elements.map(link).join('')}${t.count > t.elements.length ? `<li>and ${t.count - t.elements.length} more</li>` : ''}</ul></details>`)
            .join('');
        const bound = (i.bindings ?? [])
            .map(
                b =>
                    `<li>${esc(b.configuration.id ?? b.configuration.name)}: <b>${esc(b.parameter)}</b> = ${esc(String(b.value))}${this.editMode ? `<button data-action="removeBinding" data-config="${esc(b.configuration.id ?? b.configuration.qname)}" data-name="${esc(b.parameter)}" title="Remove this binding (a bound parameter cannot be removed)">&#x2715;</button>` : ''}</li>`,
            )
            .join('');
        const configs = (title: string, list: { qname: string; name: string }[] | undefined): string =>
            list && list.length ? `<details><summary>${title} (${list.length})</summary><ul>${list.map(link).join('')}</ul></details>` : '';
        return `<div class="fm-impact"><div class="detail-section-label">Impact</div>${lines}${gated}${bound ? `<details open><summary>Bindings (${(i.bindings ?? []).length})</summary><ul class="fm-edit-constraints">${bound}</ul></details>` : ''}${configs('Selected by', i.selectedBy)}${configs('Deselected by', i.deselectedBy)}
          <button data-impact="removal" data-feature="${esc(node.ref)}" data-subtree="${(i.descendants ?? 0) > 0}">What would removing it change?</button><div class="fm-removal" id="fm-removal"></div></div>`;
    }

    private async previewRemoval(feature: string, subtree: boolean): Promise<void> {
        const out = document.getElementById('fm-removal');
        if (!out) {
            return;
        }
        out.textContent = 'Checking…';
        const resp = await fetch('/api/feature-model/edit', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ edit: { op: 'remove', feature, subtree }, preview: true }),
        });
        const r = (await resp.json()) as EditResult;
        if (!r.delta) {
            out.textContent = r.reason ?? 'It cannot be removed.';
            return;
        }
        const lines = deltaLines(r.delta, q => this.nameOf(q));
        const after = r.delta.countsAfter;
        out.innerHTML = (lines.length ? lines.map(l => `<div>${esc(l)}</div>`).join('') : '<div>The model stays valid: nothing becomes dead, forced or void.</div>') + `<div>${after.features} features would remain.</div>`;
    }

    private async fit(): Promise<void> {
        const canvas = byId('fm-canvas');
        const roots = this.full ? rootIds(this.full) : [];
        if (viewMode(this.boxes, { width: canvas.clientWidth, height: canvas.clientHeight }) === 'roots' && roots.length > 0) {
            // Too wide to read at once: start at the first root and let the user zoom, pan and search.
            await this.dispatcher.dispatch(CenterAction.create([roots[0]], { animate: false, zoomScale: 0.6 }));
            this.toast('This model is too wide to read at once. Zoom and pan, search for a feature, or collapse subtrees.');
            return;
        }
        await this.dispatcher.dispatch(FitToScreenAction.create([], { padding: 30, maxZoom: 1.2 }));
    }

    private toggle(id: string): void {
        if (this.collapsed.has(id)) {
            this.collapsed.delete(id);
        } else {
            this.collapsed.add(id);
        }
        void this.render();
    }

    private onSelection(sel: string[], desel: string[]): void {
        for (const id of desel) {
            if (this.selected === id) {
                this.selected = null;
            }
        }
        const feature = sel.find(id => this.full && featureNodes(this.full).some(n => n.id === id));
        if (feature) {
            this.selected = feature;
        }
        this.showSelected();
        this.renderEditPanel();
    }

    private onSearch(query: string): void {
        if (!this.full) {
            return;
        }
        if (this.matrixMode) {
            this.renderMatrix();
            return;
        }
        this.matches = search(this.full, query);
        this.matchIndex = 0;
        if (this.matches.length > 0) {
            this.collapsed = revealing(this.full, this.collapsed, this.matches);
        }
        void this.render().then(() => {
            if (this.matches.length > 0) {
                void this.dispatcher.dispatch(CenterAction.create([this.matches[0].id], { animate: true }));
            }
        });
    }

    private nextMatch(step: number): void {
        if (this.matches.length === 0) {
            return;
        }
        this.matchIndex = (this.matchIndex + step + this.matches.length) % this.matches.length;
        void this.dispatcher.dispatch(CenterAction.create([this.matches[this.matchIndex].id], { animate: true }));
    }

    /** The Inspector: what the feature is, what the analysis says about it and why,
     * its constraints, then its documentation from the server's element card. */
    private showSelected(): void {
        const pane = byId('fm-selected');
        const node = this.full && this.selected ? featureNodes(this.full).find(n => n.id === this.selected) : undefined;
        if (!node || !this.full) {
            pane.innerHTML = '<p class="detail-empty">Select a feature to see what it is and why.</p>';
            return;
        }
        const a = this.analysis?.features[node.ref];
        const name = (id: string): string => featureNodes(this.full as DiagramModelSchema).find(n => n.id === id)?.name ?? id;
        const edges = this.full.children.filter(isEdgeSchema);
        const list = (kind: string, from: 'source' | 'target'): string[] =>
            edges
                .filter(e => e.kind === kind && (from === 'source' ? e.sourceId : e.targetId) === node.id)
                .map(e => name(from === 'source' ? e.targetId : e.sourceId));
        const rows: string[] = [];
        const state = a?.state ?? 'normal';
        const stateText: Record<string, string> = {
            normal: 'selectable, and optional',
            core: 'core: in every product',
            dead: 'dead: in no product',
            falseOptional: 'false-optional: declared optional but forced',
        };
        rows.push(`<div class="fm-state fm-state-${state}">${esc(stateText[state])}</div>`);
        if (a && a.reasons.length > 0) {
            rows.push('<div class="ec-title">Why</div><ul class="fm-reasons">' + a.reasons.map(r => `<li>${esc(r)}</li>`).join('') + '</ul>');
        }
        const m = node.feature;
        if (m) {
            rows.push(`<div class="fm-meta">${node.isAbstract ? 'abstract, ' : ''}${m.mandatory ? 'mandatory' : 'optional'} member${m.childCount ? `, children grouped as ${m.group}` : ''}</div>`);
        }
        const req = list('requires', 'source');
        const reqBy = list('requires', 'target');
        const ex = [...list('excludes', 'source'), ...list('excludes', 'target')];
        if (req.length) {
            rows.push(`<div class="fm-meta">requires ${req.map(esc).join(', ')}</div>`);
        }
        if (reqBy.length) {
            rows.push(`<div class="fm-meta">required by ${reqBy.map(esc).join(', ')}</div>`);
        }
        if (ex.length) {
            rows.push(`<div class="fm-meta">excludes ${ex.map(esc).join(', ')}</div>`);
        }
        // The element card below carries the name, type, path and documentation; this is what the analysis adds.
        pane.innerHTML = `${rows.join('')}<div id="fm-card"></div>`;
        const wanted = node.id;
        void fetch('/api/feature-model/impact?feature=' + encodeURIComponent(node.ref))
            .then(r => r.json() as Promise<ImpactResult>)
            .then(impact => {
                const card = document.getElementById('fm-card');
                if (card && this.selected === wanted && impact.found) {
                    card.insertAdjacentHTML('beforebegin', this.renderImpact(impact, node));
                    (window as unknown as { htmx?: { process(el: Element): void } }).htmx?.process(byId('fm-selected'));
                }
            });
        const url = '/ui/element-card/' + node.ref.split('::').map(encodeURIComponent).join('/');
        void fetch(url)
            .then(r => r.text())
            .then(html => {
                const card = document.getElementById('fm-card');
                if (card && this.selected === wanted) {
                    card.innerHTML = html;
                }
            });
    }
}

window.addEventListener('DOMContentLoaded', () => {
    if (document.getElementById(HOST)) {
        (window as unknown as { FeaturePage: FeaturePage }).FeaturePage = new FeaturePage();
    }
});
