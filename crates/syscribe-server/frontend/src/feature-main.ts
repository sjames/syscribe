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
    collapseBelow,
    configCounts,
    ConfigureResult,
    configurationFields,
    configurationPackage,
    describeConflict,
    FeatureAnalysis,
    featureNodes,
    nextChoice,
    productsText,
    revealing,
    search,
    StoredConfiguration,
    summaryLines,
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
    private readonly dispatcher: IActionDispatcher;
    private readonly source: LocalModelSource;

    constructor() {
        const container = createDiagramContainer(HOST, {
            // The diagram is a view of the model; a dragged feature snaps back.
            onMoveFinished: () => void this.render(),
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
        byId('fm-configure').addEventListener('click', () => void this.setConfigMode(!this.configMode));
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
        if (fit || this.firstRender) {
            this.firstRender = false;
            await this.fit();
        }
        this.showSelected();
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

    private async fit(): Promise<void> {
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
    }

    private onSearch(query: string): void {
        if (!this.full) {
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
            rows.push(`<div class="fm-meta">${m.mandatory ? 'mandatory' : 'optional'} member${m.childCount ? `, children grouped as ${m.group}` : ''}</div>`);
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
        pane.innerHTML = `<div class="detail-name">${esc(node.name)}</div><div class="detail-qname">${esc(node.ref)}</div>${rows.join('')}<div id="fm-card"></div>`;
        const url = '/ui/element-card/' + node.ref.split('::').map(encodeURIComponent).join('/');
        const wanted = node.id;
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
