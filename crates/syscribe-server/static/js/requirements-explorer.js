// Requirements Explorer client (GH #269, REQ-TRS-REQEXPL-001). Plain vanilla script, no build step.
// Fetches /api/req-graph, lays the neighbourhood out in layers by hop distance and draws it as SVG.
// All model text is written with textContent / attributes — never innerHTML — except the server's own
// element-card fragment, which the server renders and escapes.
(function () {
  'use strict';

  var NODE_W = 190, NODE_H = 44, GAP_X = 96, GAP_Y = 22, PAD = 24;
  var EDGE_KINDS = ['derivedFrom', 'satisfies', 'verifies', 'allocatedTo', 'refines', 'supersedes',
    'derivedFromSafetyGoal', 'breakdownAdr', 'blockedBy', 'covers', 'analyses', 'runsOn', 'achieves', 'evidence', 'appliesWhen',
    'hazardRef', 'hazardousEvents', 'threatScenarios', 'mitigatedBy', 'derivedFromCybersecurityGoal', 'confirms', 'implementedBy', 'threatRef', 'relatedSafetyGoal',
    'damageScenarios', 'assets', 'implementsGoals', 'affectedElements', 'supports', 'topEvent', 'ftaRef', 'assetOwner'];
  var FILL = { verified: '#e3f4e5', planned: '#fff3d6', unverified: '#fde4e1', na: '#eceff4' };
  var STROKE = { verified: '#2e7d32', planned: '#b7791f', unverified: '#c0392b', na: '#7a869a' };

  var VM_LABELS = ['Stakeholder requirements', 'System requirements', 'Other requirements', 'Architecture', 'Tests', 'Other elements'];

  // The V-model column of a node (0 stakeholder .. 5 everything else); a pure function of the node.
  function vmodelColumn(n) {
    if (n.type === 'Requirement') return n.reqClass === 'stakeholder' ? 0 : n.reqClass === 'system' ? 1 : 2;
    if (CATEGORIES.architecture.indexOf(n.type) >= 0) return 3;
    if (CATEGORIES.tests.indexOf(n.type) >= 0) return 4;
    return 5;
  }

  // Pure layout: layers by hop distance from the root over the undirected edges. Nodes unreachable from
  // the root (the API never returns any) go one column past the last. Deterministic: ids sorted per column.
  // `colOf(node)` (optional) assigns raw columns instead (a non-finite value counts as 0); only occupied
  // columns are used, in order, and `labels[raw]` names the header of each (blank when absent).
  function layoutGraph(g, colOf, labels) {
    var ids = g.nodes.map(function (n) { return n.id; });
    var adj = {};
    ids.forEach(function (id) { adj[id] = []; });
    (g.edges || []).forEach(function (e) {
      if (adj[e.from] && adj[e.to]) { adj[e.from].push(e.to); adj[e.to].push(e.from); }
    });
    var col = {};
    var queue = [];
    if (g.root && adj[g.root]) { col[g.root] = 0; queue.push(g.root); }
    while (queue.length) {
      var cur = queue.shift();
      adj[cur].slice().sort().forEach(function (n) {
        if (col[n] === undefined) { col[n] = col[cur] + 1; queue.push(n); }
      });
    }
    var maxCol = Object.keys(col).reduce(function (m, k) { return Math.max(m, col[k]); }, 0);
    ids.forEach(function (id) { if (col[id] === undefined) col[id] = maxCol + 1; });
    var headers;
    if (colOf) {
      var raws = [];
      var raw = function (n) { var r = Number(colOf(n)); return isFinite(r) ? r : 0; };
      g.nodes.forEach(function (n) { var r = raw(n); if (raws.indexOf(r) < 0) raws.push(r); });
      raws.sort(function (a, b) { return a - b; });
      var rank = {};
      raws.forEach(function (r, i) { rank[r] = i; });
      g.nodes.forEach(function (n) { col[n.id] = rank[raw(n)]; });
      headers = raws.map(function (r, i) { return { col: i, label: (labels && labels[r]) || '' }; });
    }
    var byCol = {};
    g.nodes.forEach(function (n) { (byCol[col[n.id]] = byCol[col[n.id]] || []).push(n); });
    var cols = Object.keys(byCol).map(Number).sort(function (a, b) { return a - b; });
    var tallest = cols.reduce(function (m, c) { return Math.max(m, byCol[c].length); }, 1);
    var height = PAD * 2 + tallest * NODE_H + (tallest - 1) * GAP_Y;
    var nodes = [];
    cols.forEach(function (c) {
      var list = byCol[c].slice().sort(function (a, b) { return a.id < b.id ? -1 : a.id > b.id ? 1 : 0; });
      var colHeight = list.length * NODE_H + (list.length - 1) * GAP_Y;
      var top = PAD + (height - PAD * 2 - colHeight) / 2;
      list.forEach(function (n, i) {
        var copy = {};
        Object.keys(n).forEach(function (k) { copy[k] = n[k]; });
        copy.col = c; copy.row = i;
        copy.x = PAD + c * (NODE_W + GAP_X);
        copy.y = top + i * (NODE_H + GAP_Y);
        copy.w = NODE_W; copy.h = NODE_H;
        nodes.push(copy);
      });
    });
    var width = PAD * 2 + (cols.length ? cols[cols.length - 1] + 1 : 1) * NODE_W + (cols.length ? cols[cols.length - 1] : 0) * GAP_X;
    var out = { nodes: nodes, edges: g.edges || [], width: width, height: height };
    if (headers) out.headers = headers;
    return out;
  }

  // Pure filter: keeps the root and every node matching ALL non-empty criteria (types, verification, asil);
  // drops edges touching a hidden node. An empty or missing criterion restricts nothing.
  // The level a node is rated at: decomposition notation `B(D)` counts as B; case-insensitive.
  function effectiveAsil(a) { return a ? String(a).split('(')[0].trim().toUpperCase() : undefined; }

  function filterGraph(g, f) {
    f = f || {};
    function ok(list, v) { return !list || !list.length || list.indexOf(v) >= 0; }
    var keep = {};
    g.nodes.forEach(function (n) {
      if (n.id === g.root || (ok(f.types, n.type) && ok(f.verification, n.verification || 'na') && ok(f.asil, effectiveAsil(n.asil)))) keep[n.id] = true;
    });
    // Only what is still connected to the root through kept nodes is drawn: hiding an intermediate node
    // must not leave the nodes behind it floating in a spurious column.
    var adj = {};
    (g.edges || []).forEach(function (e) {
      if (!keep[e.from] || !keep[e.to]) return;
      (adj[e.from] = adj[e.from] || []).push(e.to);
      (adj[e.to] = adj[e.to] || []).push(e.from);
    });
    var reach = {};
    var stack = g.root && keep[g.root] ? [g.root] : [];
    if (stack.length) reach[g.root] = true;
    while (stack.length) {
      (adj[stack.pop()] || []).forEach(function (n) { if (!reach[n]) { reach[n] = true; stack.push(n); } });
    }
    var nodes = g.nodes.filter(function (n) { return reach[n.id]; });
    var edges = (g.edges || []).filter(function (e) { return reach[e.from] && reach[e.to]; });
    var out = {};
    Object.keys(g).forEach(function (k) { out[k] = g[k]; });
    out.nodes = nodes; out.edges = edges; out.hidden = g.nodes.length - nodes.length;
    return out;
  }

  var CATEGORIES = {
    tests: ['TestCase', 'TestPlan', 'TestEnvironment', 'VerificationCase', 'VerificationCaseDef'],
    architecture: ['PartDef', 'Part', 'ItemDef', 'PortDef', 'InterfaceDef', 'ConnectionDef', 'Allocation', 'AllocationDef', 'ActionDef', 'StateDef',
      'Item', 'Port', 'Interface', 'Connection', 'Action', 'State'],
    features: ['FeatureDef', 'Configuration', 'FeatureModel'],
    safety: ['HazardousEvent', 'SafetyGoal', 'SafetyMechanism', 'ConfirmationMeasure', 'DependentFailureAnalysis', 'AssumptionOfUse', 'Argument',
      'FaultTree', 'FaultTreeGate', 'FaultTreeEvent', 'FMEASheet', 'FMEAEntry'],
    security: ['DamageScenario', 'ThreatScenario', 'CybersecurityGoal', 'SecurityControl', 'VulnerabilityReport', 'Asset', 'AttackTree',
      'AttackTreeGate', 'AttackStep', 'Zone', 'Conduit', 'TARASheet']
  };

  function categoryOf(type) {
    var found;
    Object.keys(CATEGORIES).forEach(function (c) { if (CATEGORIES[c].indexOf(type) >= 0) found = c; });
    return found;
  }

  // Pure: every node and edge on a shortest path (either edge direction) from `start` to a node of the
  // category. The start is never a target. Unknown category, unknown start or no match: empty result.
  function tracePath(g, start, category) {
    var empty = { nodes: [], edges: [], targets: [] };
    var types = CATEGORIES[category];
    if (!types) return empty;
    var ids = {};
    g.nodes.forEach(function (n) { ids[n.id] = n; });
    if (!ids[start]) return empty;
    var adj = {};
    (g.edges || []).forEach(function (e, i) {
      if (!ids[e.from] || !ids[e.to]) return;
      (adj[e.from] = adj[e.from] || []).push({ to: e.to, i: i });
      (adj[e.to] = adj[e.to] || []).push({ to: e.from, i: i });
    });
    var dist = {}; dist[start] = 0;
    var queue = [start];
    while (queue.length) {
      var u = queue.shift();
      (adj[u] || []).forEach(function (a) { if (dist[a.to] === undefined) { dist[a.to] = dist[u] + 1; queue.push(a.to); } });
    }
    var targets = g.nodes.filter(function (n) { return n.id !== start && dist[n.id] !== undefined && types.indexOf(n.type) >= 0; }).map(function (n) { return n.id; });
    var onNode = {}, onEdge = {};
    targets.forEach(function (t) { onNode[t] = true; });
    // Walk back from the targets by decreasing distance, marking every predecessor on a shortest path.
    var order = targets.slice();
    while (order.length) {
      order.sort(function (a, b) { return dist[b] - dist[a]; });
      var v = order.shift();
      (adj[v] || []).forEach(function (a) {
        if (dist[a.to] === dist[v] - 1) {
          onEdge[a.i] = true;
          if (!onNode[a.to]) { onNode[a.to] = true; order.push(a.to); }
        }
      });
    }
    return {
      nodes: g.nodes.filter(function (n) { return onNode[n.id]; }).map(function (n) { return n.id; }),
      edges: Object.keys(onEdge).map(Number).sort(function (a, b) { return a - b; }),
      targets: targets
    };
  }

  var VIEW_LAYOUTS = ['hops', 'v-model'];
  var VIEW_MODES = ['graph', 'table', 'matrix'];
  var VIEW_DEFAULT = { layout: 'hops', view: 'graph', trace: '', cols: 'tests' };

  // Pure: the view state of a query string; anything outside its vocabulary falls back to the default.
  function parseViewState(search) {
    var p = new URLSearchParams(search || '');
    function pick(key, allowed, dflt) { var v = p.get(key); return v !== null && allowed.indexOf(v) >= 0 ? v : dflt; }
    var cats = Object.keys(CATEGORIES);
    return {
      layout: pick('layout', VIEW_LAYOUTS, VIEW_DEFAULT.layout),
      view: pick('view', VIEW_MODES, VIEW_DEFAULT.view),
      trace: pick('trace', cats, VIEW_DEFAULT.trace),
      cols: pick('cols', cats, VIEW_DEFAULT.cols)
    };
  }

  // Pure inverse: defaults are omitted.
  function viewStateQuery(s) {
    var p = new URLSearchParams();
    ['layout', 'view', 'trace', 'cols'].forEach(function (k) { if (s[k] && s[k] !== VIEW_DEFAULT[k]) p.set(k, s[k]); });
    return p.toString();
  }

  // Fixed paths only: model text never reaches an href.
  function jumpLinks(n) {
    if (n.type === 'FeatureDef' || n.type === 'Configuration' || n.type === 'FeatureModel') return [{ label: 'Open the feature model', href: '/features' }];
    if (n.type === 'PlanningItem') return [{ label: 'Open the planning board', href: '/planning' }];
    return [];
  }

  // Pure: the drawn graph as table rows; `hop` is the undirected distance from the root (null if unreachable).
  function tableRows(g) {
    var adj = {};
    g.nodes.forEach(function (n) { adj[n.id] = []; });
    (g.edges || []).forEach(function (e) { if (adj[e.from] && adj[e.to]) { adj[e.from].push(e.to); adj[e.to].push(e.from); } });
    var hop = {};
    var queue = [];
    if (g.root && adj[g.root]) { hop[g.root] = 0; queue.push(g.root); }
    while (queue.length) {
      var u = queue.shift();
      adj[u].forEach(function (n) { if (hop[n] === undefined) { hop[n] = hop[u] + 1; queue.push(n); } });
    }
    return {
      elements: g.nodes.map(function (n) {
        return { id: n.id, qname: n.qname, type: n.type || '', name: n.name || '', status: n.status || '', asil: n.asil || '', verification: n.verification || '', hop: hop[n.id] === undefined ? null : hop[n.id] };
      }),
      relations: (g.edges || []).map(function (e) { return { from: e.from, to: e.to, kind: e.kind }; })
    };
  }

  // Pure: Requirement rows against the nodes of one category; a cell holds the sorted, distinct kinds of the
  // direct edges between row and column (either direction). Rows/columns with no cell are kept.
  function matrixModel(g, category) {
    var types = CATEGORIES[category];
    if (!types) return { rows: [], cols: [], cells: {}, noneIn: {} };
    function byId(a, b) { return a < b ? -1 : a > b ? 1 : 0; }
    var rows = g.nodes.filter(function (n) { return n.type === 'Requirement'; }).map(function (n) { return n.id; }).sort(byId);
    var cols = g.nodes.filter(function (n) { return types.indexOf(n.type) >= 0; }).map(function (n) { return n.id; }).sort(byId);
    var isRow = {}, isCol = {};
    rows.forEach(function (r) { isRow[r] = true; });
    cols.forEach(function (c) { isCol[c] = true; });
    var cells = {};
    (g.edges || []).forEach(function (e) {
      [[e.from, e.to], [e.to, e.from]].forEach(function (p) {
        if (isRow[p[0]] && isCol[p[1]] && p[0] !== p[1]) {
          var k = p[0] + '|' + p[1];
          var kind = e.kind || '(unnamed)';
          cells[k] = cells[k] || [];
          if (cells[k].indexOf(kind) < 0) cells[k].push(kind);
        }
      });
    });
    Object.keys(cells).forEach(function (k) { cells[k].sort(); });
    var noneIn = {};
    rows.forEach(function (r) { noneIn[r] = !cols.some(function (c) { return cells[r + '|' + c]; }); });
    return { rows: rows, cols: cols, cells: cells, noneIn: noneIn };
  }

  // RFC 4180 quoting; a cell a spreadsheet would evaluate as a formula gets a leading apostrophe.
  function csvCell(v) {
    var t = v === undefined || v === null ? '' : String(v);
    if (/^[=+\-@\t\r]/.test(t)) t = "'" + t;
    return /[",\n\r]/.test(t) ? '"' + t.replace(/"/g, '""') + '"' : t;
  }

  // One table, one column set: `record` says whether a row is an element or a relation. CRLF line ends.
  function toCsv(g) {
    var t = tableRows(g);
    var out = ['record,id,type,name,status,asil,verification,hop,from,to,kind'];
    t.elements.forEach(function (r) { out.push(['element', r.id, r.type, r.name, r.status, r.asil, r.verification, r.hop, '', '', ''].map(csvCell).join(',')); });
    t.relations.forEach(function (r) { out.push(['relation', '', '', '', '', '', '', '', r.from, r.to, r.kind].map(csvCell).join(',')); });
    return out.join('\r\n') + '\r\n';
  }

  function exportName(root, ext) {
    var safe = String(root || '').replace(/[^A-Za-z0-9._-]/g, '_') || 'graph';
    return 'req-graph-' + safe + '.' + ext;
  }

  function el(name, attrs, text) {
    var e = document.createElementNS('http://www.w3.org/2000/svg', name);
    Object.keys(attrs || {}).forEach(function (k) { e.setAttribute(k, attrs[k]); });
    if (text !== undefined) e.textContent = text;
    return e;
  }

  function truncate(s, n) { s = s || ''; return s.length > n ? s.slice(0, n - 1) + '…' : s; }

  var state = { focus: '', depth: 1, config: '', kinds: null, selected: null, lastGraph: null, filters: {}, trace: '', drawn: null, view: 'graph', layout: 'hops', cols: 'tests' };
  var root, svg, statusEl, detailEl;

  function setStatus(t) { if (statusEl) statusEl.textContent = t || ''; }

  function draw(raw) {
    state.lastGraph = raw;
    syncTypeFilters(raw);
    var graph = filterGraph(raw, state.filters);
    if (state.selected && !graph.nodes.some(function (n) { return n.id === state.selected; })) resetDetail();
    var lay = layoutGraph(graph, state.layout === 'v-model' ? vmodelColumn : undefined, VM_LABELS);
    var tr = null;
    if (state.trace) {
      var start = state.selected && graph.nodes.some(function (n) { return n.id === state.selected; }) ? state.selected : graph.root;
      tr = tracePath(graph, start, state.trace);
      tr.nodeSet = {}; tr.nodes.forEach(function (i) { tr.nodeSet[i] = true; });
      tr.edgeSet = {}; tr.edges.forEach(function (i) { tr.edgeSet[i] = true; });
      tr.targetSet = {}; tr.targets.forEach(function (i) { tr.targetSet[i] = true; });
    }
    while (svg.firstChild) svg.removeChild(svg.firstChild);
    svg.setAttribute('width', lay.width);
    svg.setAttribute('height', lay.height);
    svg.setAttribute('viewBox', '0 0 ' + lay.width + ' ' + lay.height);
    var defs = el('defs');
    var marker = el('marker', { id: 'rg-arrow', viewBox: '0 0 10 10', refX: '9', refY: '5', markerWidth: '7', markerHeight: '7', orient: 'auto-start-reverse' });
    marker.appendChild(el('path', { d: 'M0,0 L10,5 L0,10 z', fill: '#7a869a' }));
    defs.appendChild(marker);
    svg.appendChild(defs);
    (lay.headers || []).forEach(function (h) {
      svg.appendChild(el('text', { x: PAD + h.col * (NODE_W + GAP_X), y: PAD - 8, class: 'rg-colhead' }, h.label));
    });
    var pos = {};
    lay.nodes.forEach(function (n) { pos[n.id] = n; });
    var labels = [];
    var seenPair = {};
    lay.edges.forEach(function (e, ei) {
      var a = pos[e.from], b = pos[e.to];
      if (!a || !b) return;
      var x1, y1, x2, y2, d;
      if (a.col === b.col) {
        x1 = a.x + a.w; y1 = a.y + a.h / 2; x2 = b.x + b.w; y2 = b.y + b.h / 2;
        var bulge = 40;
        d = 'M' + x1 + ',' + y1 + ' C' + (x1 + bulge) + ',' + y1 + ' ' + (x2 + bulge) + ',' + y2 + ' ' + x2 + ',' + y2;
      } else {
        var left = a.col < b.col ? a : b, rightN = a.col < b.col ? b : a;
        x1 = left.x + left.w; y1 = left.y + left.h / 2; x2 = rightN.x; y2 = rightN.y + rightN.h / 2;
        // arrowhead at the edge's target end
        var fwd = a === left;
        var mx = (x1 + x2) / 2;
        d = fwd ? 'M' + x1 + ',' + y1 + ' C' + mx + ',' + y1 + ' ' + mx + ',' + y2 + ' ' + x2 + ',' + y2
                : 'M' + x2 + ',' + y2 + ' C' + mx + ',' + y2 + ' ' + mx + ',' + y1 + ' ' + x1 + ',' + y1;
      }
      var path = el('path', { d: d, class: 'rg-edge' + (tr ? (tr.edgeSet[ei] ? ' hl' : ' dim') : ''), 'marker-end': 'url(#rg-arrow)', 'data-kind': e.kind });
      path.appendChild(el('title', {}, e.from + ' → ' + e.to + ' (' + e.kind + ')'));
      svg.appendChild(path);
      var pk = [e.from, e.to].sort().join('|');
      var k = seenPair[pk] = (seenPair[pk] || 0) + 1;
      labels.push(el('text', { x: Math.abs(a.col - b.col) > 1 ? x1 + GAP_X / 2 : (x1 + x2) / 2 + (a.col === b.col ? 44 : 0), y: Math.abs(a.col - b.col) > 1 ? y1 - 4 + (k - 1) * 12 : (y1 + y2) / 2 - 4 + (k - 1) * 12, 'text-anchor': 'middle', class: 'rg-edge-label' + (tr && !tr.edgeSet[ei] ? ' dim' : '') }, e.kind));
    });
    lay.nodes.forEach(function (n) {
      var v = n.verification || 'na';
      var vlabel = { verified: 'verified', planned: 'verification planned', unverified: 'not verified', na: '' }[v] || '';
      var summary = (n.type || '') + ' ' + n.id + (n.name ? ' — ' + n.name : '') + (n.status ? ' [' + n.status + ']' : '') + (vlabel ? ', ' + vlabel : '') + (tr && tr.nodeSet[n.id] ? (tr.targetSet[n.id] ? ', trace target' : ', on trace') : '') + (state.layout === 'v-model' ? ', ' + VM_LABELS[vmodelColumn(n)] : '');
      var g = el('g', { class: 'rg-node' + (n.root ? ' root' : '') + (tr ? (tr.targetSet[n.id] ? ' target hl' : tr.nodeSet[n.id] ? ' hl' : ' dim') : '') + (state.selected === n.id ? ' selected' : ''), 'data-id': n.id, tabindex: '0', role: 'button', 'aria-label': summary, 'aria-pressed': state.selected === n.id ? 'true' : 'false' });
      g.appendChild(el('title', {}, summary));
      g.appendChild(el('rect', { x: n.x, y: n.y, width: n.w, height: n.h, rx: 6, fill: FILL[v] || FILL.na, stroke: STROKE[v] || STROKE.na }));
      g.appendChild(el('text', { x: n.x + 8, y: n.y + 18 }, truncate(n.id, 28)));
      var sub = [n.type, n.asil ? 'ASIL ' + n.asil : null, n.status, v === 'verified' ? '✓' : v === 'unverified' ? '✗' : v === 'planned' ? '◐' : null].filter(Boolean).join(' · ');
      g.appendChild(el('text', { x: n.x + 8, y: n.y + 34, class: 'rg-sub' }, truncate(sub, 34)));
      g.addEventListener('click', function () { select(n); });
      g.addEventListener('keydown', function (ev) { if (ev.key === 'Enter' || ev.key === ' ') { ev.preventDefault(); select(n); } });
      svg.appendChild(g);
    });
    renderTable(graph, tr);
    renderMatrix(graph);
    state.drawn = graph; // only once the SVG and the table both show it
    labels.forEach(function (t) { svg.appendChild(t); }); // above the nodes, with a halo (see the stylesheet)
    setStatus(graph.nodes.length + ' element(s), ' + (graph.edges || []).length + ' relation(s)' + (graph.hidden ? ', ' + graph.hidden + ' hidden by filters' : '') + (tr ? (tr.targets.length ? ', ' + tr.targets.length + ' ' + state.trace + ' element(s) on the trace' : ', no ' + state.trace + ' element in view — raise the depth or tick more relations') : '') + (graph.truncated ? ' — truncated at the server node limit — lower the depth or untick relations' : ''));
  }

  function cell(row, text, tag) {
    var c = document.createElement(tag || 'td');
    c.textContent = text === null || text === undefined ? '' : String(text);
    row.appendChild(c);
    return c;
  }

  // The table lens: the same drawn graph as two tables; id cells are buttons, so it is the keyboard equivalent.
  function renderTable(graph, tr) {
    var host = document.getElementById('req-table');
    if (!host) return;
    host.textContent = '';
    var t = tableRows(graph);
    var el1 = document.createElement('table');
    var cap = document.createElement('caption'); cap.textContent = 'Elements'; el1.appendChild(cap);
    var thead = document.createElement('thead');
    var head = document.createElement('tr');
    ['Id', 'Type', 'Name', 'Status', 'ASIL', 'Verification', 'Hop', 'Trace'].forEach(function (h) { cell(head, h, 'th').setAttribute('scope', 'col'); });
    thead.appendChild(head);
    el1.appendChild(thead);
    var tbody = document.createElement('tbody');
    el1.appendChild(tbody);
    t.elements.forEach(function (r) {
      var tr1 = document.createElement('tr');
      var idc = document.createElement('td');
      var b = document.createElement('button');
      b.type = 'button'; b.textContent = r.id;
      b.addEventListener('click', function () { select(r); });
      idc.appendChild(b); tr1.appendChild(idc);
      [r.type, r.name, r.status, r.asil, r.verification, r.hop].forEach(function (v) { cell(tr1, v); });
      cell(tr1, tr ? (tr.targetSet[r.id] ? 'target' : tr.nodeSet[r.id] ? 'on trace' : '') : '');
      if (state.selected === r.id) b.setAttribute('aria-current', 'true');
      tbody.appendChild(tr1);
    });
    host.appendChild(el1);
    var rel = document.createElement('table');
    var cap2 = document.createElement('caption'); cap2.textContent = 'Relations'; rel.appendChild(cap2);
    var th2 = document.createElement('thead');
    var h2 = document.createElement('tr');
    ['From', 'To', 'Kind'].forEach(function (h) { cell(h2, h, 'th').setAttribute('scope', 'col'); });
    th2.appendChild(h2);
    rel.appendChild(th2);
    var tb2 = document.createElement('tbody');
    rel.appendChild(tb2);
    t.relations.forEach(function (r) { var x = document.createElement('tr'); cell(x, r.from).setAttribute('scope', 'row'); cell(x, r.to); cell(x, r.kind); tb2.appendChild(x); });
    host.appendChild(rel);
  }

  function renderMatrix(graph) {
    var host = document.getElementById('req-matrix');
    if (!host) return;
    host.textContent = '';
    var m = matrixModel(graph, state.cols);
    var t = document.createElement('table');
    var cap = document.createElement('caption');
    cap.textContent = 'Requirements by ' + state.cols;
    t.appendChild(cap);
    var thead = document.createElement('thead');
    var hr = document.createElement('tr');
    cell(hr, 'Requirement', 'th').setAttribute('scope', 'col');
    m.cols.forEach(function (c) { cell(hr, c, 'th').setAttribute('scope', 'col'); });
    cell(hr, 'Coverage', 'th').setAttribute('scope', 'col');
    thead.appendChild(hr);
    t.appendChild(thead);
    var tbody = document.createElement('tbody');
    var byId = {};
    graph.nodes.forEach(function (n) { byId[n.id] = n; });
    function idButton(td, id) {
      var b = document.createElement('button');
      b.type = 'button'; b.textContent = id;
      b.addEventListener('click', function () { select(byId[id]); });
      td.appendChild(b);
    }
    m.rows.forEach(function (r) {
      var tr1 = document.createElement('tr');
      var th = document.createElement('th');
      th.setAttribute('scope', 'row');
      idButton(th, r);
      tr1.appendChild(th);
      m.cols.forEach(function (c) { cell(tr1, (m.cells[r + '|' + c] || []).join(', ')); });
      cell(tr1, m.noneIn[r] ? 'none' : '');
      tbody.appendChild(tr1);
    });
    t.appendChild(tbody);
    if (!m.rows.length) {
      var p = document.createElement('p');
      p.textContent = 'No requirement is in view.';
      host.appendChild(p);
    }
    host.appendChild(t);
  }

  function setView(v) {
    state.view = v;
    var host = document.getElementById('req-table');
    var btn = document.getElementById('req-view-table');
    if (host) host.hidden = v !== 'table';
    var mhost = document.getElementById('req-matrix');
    if (mhost) mhost.hidden = v !== 'matrix';
    var mcols = document.getElementById('req-matrix-cols');
    if (mcols && mcols.parentNode) mcols.parentNode.hidden = v !== 'matrix';
    var canvas = document.querySelector('.req-canvas');
    if (canvas) canvas.hidden = v !== 'graph';
    if (btn) { btn.setAttribute('aria-pressed', v === 'table' ? 'true' : 'false'); }
    var mbtn = document.getElementById('req-view-matrix');
    if (mbtn) mbtn.setAttribute('aria-pressed', v === 'matrix' ? 'true' : 'false');
  }

  // The page styles the graph from its stylesheet; a standalone SVG needs the same rules inline.
  var SVG_STYLE = '.rg-node text{font:12px sans-serif}.rg-node .rg-sub{font-size:10px;fill:#555}.rg-node rect{stroke-width:1.5}' +
    '.rg-node.root rect{stroke-width:3.5}.rg-node.selected rect{stroke:#1a4fd6;stroke-width:3}.rg-edge{fill:none;stroke:#7a869a;stroke-width:1.4}' +
    '.rg-edge-label{font:10px sans-serif;fill:#2b3550;paint-order:stroke;stroke:#fff;stroke-width:3px;stroke-linejoin:round}' +
    '.rg-colhead{font:bold 11px sans-serif;fill:#2b3550}.dim{opacity:.18}.rg-edge.hl{stroke-width:2.5px}.rg-node.target rect{stroke-width:3px;stroke-dasharray:5 2}';

  function svgText() {
    var c = svg.cloneNode(true);
    var st = document.createElementNS('http://www.w3.org/2000/svg', 'style');
    st.textContent = SVG_STYLE;
    var bg = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
    bg.setAttribute('width', '100%'); bg.setAttribute('height', '100%'); bg.setAttribute('fill', '#fff');
    c.insertBefore(bg, c.firstChild);
    c.insertBefore(st, c.firstChild);
    c.style.display = '';
    return new XMLSerializer().serializeToString(c);
  }

  function download(name, mime, text) {
    var blob = new Blob([text], { type: mime });
    var url = URL.createObjectURL(blob);
    var a = document.createElement('a');
    a.href = url; a.download = name;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    setTimeout(function () { URL.revokeObjectURL(url); }, 1000);
  }

  function exportAs(kind) {
    var g = state.drawn;
    if (!g) { setStatus('Nothing to export yet.'); return; }
    if (kind === 'json') download(exportName(g.root, 'json'), 'application/json', JSON.stringify(g, null, 2) + '\n');
    else if (kind === 'csv') download(exportName(g.root, 'csv'), 'text/csv;charset=utf-8', '\uFEFF' + toCsv(g));
    else if (kind === 'svg') download(exportName(g.root, 'svg'), 'image/svg+xml', svgText());
  }

  function qnamePath(q) { return q.split('::').map(encodeURIComponent).join('/'); }

  function select(n) {
    state.selected = n.id;
    Array.prototype.forEach.call(svg.querySelectorAll('.rg-node'), function (g) {
      var on = g.getAttribute('data-id') === n.id;
      g.classList.toggle('selected', on);
      g.setAttribute('aria-pressed', on ? 'true' : 'false');
    });
    if (state.trace && state.lastGraph) {
      draw(state.lastGraph); // the trace starts at the selection; the redraw rebuilds the nodes, so give focus back
      var again = svg.querySelector('.rg-node[data-id="' + String(n.id).replace(/["\\]/g, '\\$&') + '"]');
      if (state.view === 'table' || state.view === 'matrix') {
        var bs = document.querySelectorAll(state.view === 'table' ? '#req-table tbody button' : '#req-matrix tbody button');
        for (var bi = 0; bi < bs.length; bi++) if (bs[bi].textContent === String(n.id)) { bs[bi].focus(); break; }
      } else if (again && again.focus) again.focus();
    }
    detailEl.textContent = '';
    var bar = document.createElement('p');
    var btn = document.createElement('button');
    btn.type = 'button';
    btn.textContent = 'Focus here';
    btn.addEventListener('click', function () { focusOn(n.id, true); });
    bar.appendChild(btn);
    jumpLinks(n).forEach(function (l) {
      var a = document.createElement('a');
      a.href = l.href; a.textContent = l.label; a.className = 'req-jump';
      bar.appendChild(document.createTextNode(' '));
      bar.appendChild(a);
    });
    detailEl.appendChild(bar);
    var holder = document.createElement('div');
    detailEl.appendChild(holder);
    fetch('/ui/element-card/' + qnamePath(n.qname)).then(function (r) { return r.ok ? r.text() : Promise.reject(r.status); })
      .then(function (html) {
        holder.innerHTML = html; // the server's own, escaped fragment
        // The card carries htmx buttons and may carry a mermaid block: wire them as the diagram panel does.
        if (window.htmx && window.htmx.process) window.htmx.process(holder);
        var mm = holder.querySelectorAll('pre.mermaid');
        if (window.mermaid && mm.length) { try { window.mermaid.run({ nodes: mm }); } catch (e) { /* leave the source text */ } }
      })
      .catch(function () { holder.textContent = 'No element card for ' + n.qname + '.'; });
  }

  function currentKinds() {
    var boxes = document.querySelectorAll('#req-kinds input[type=checkbox]');
    var on = [];
    Array.prototype.forEach.call(boxes, function (b) { if (b.checked) on.push(b.value); });
    return on.length === boxes.length ? null : on;
  }

  function query() {
    var p = new URLSearchParams();
    p.set('root', state.focus);
    p.set('depth', String(state.depth));
    if (state.config) p.set('config', state.config);
    var kinds = currentKinds();
    if (kinds) p.set('edges', kinds.join(','));
    return p;
  }

  var seq = 0;

  function clearGraph() {
    while (svg.firstChild) svg.removeChild(svg.firstChild);
    state.lastGraph = null; state.drawn = null;
    ['req-table', 'req-matrix'].forEach(function (id) {
      var host = document.getElementById(id);
      if (host) host.textContent = '';
    });
  }

  function load() {
    var my = ++seq; // only the newest request may draw: an older response arriving late is dropped
    if (!state.focus) { clearGraph(); setStatus('Enter a requirement id or qualified name.'); return Promise.resolve(); }
    setStatus('Loading…');
    return fetch('/api/req-graph?' + query().toString())
      .then(function (r) {
        return r.text().then(function (t) {
          var j = null;
          try { j = JSON.parse(t); } catch (e) { /* a plain-text or HTML error body */ }
          return { ok: r.ok && j !== null, body: j, text: t, status: r.status };
        });
      })
      .then(function (res) {
        if (my !== seq) return;
        if (!res.ok) {
          clearGraph();
          setStatus((res.body && res.body.error) || ('Request failed (' + res.status + '): ' + (res.text || '').slice(0, 160)));
          return;
        }
        try { draw(res.body); } catch (e) { setStatus('Could not draw the graph: ' + e.message); }
      }, function () { if (my === seq) setStatus('Request failed (network).'); });
  }

  function urlFor() {
    var p = new URLSearchParams();
    p.set('focus', state.focus);
    p.set('depth', String(state.depth));
    if (state.config) p.set('config', state.config);
    var vq = viewStateQuery(state);
    return '/requirements?' + p.toString() + (vq ? '&' + vq : '') + (location.hash || '');
  }

  function snapshot() {
    return { focus: state.focus, depth: state.depth, config: state.config, layout: state.layout, view: state.view, trace: state.trace, cols: state.cols };
  }

  // View controls change the current entry (Back should not step through every toggle).
  function syncUrl() { history.replaceState(snapshot(), '', urlFor()); }

  // Apply a (parsed or remembered) view state to the state object and the controls.
  function applyView(v, noDraw) {
    state.layout = v.layout || VIEW_DEFAULT.layout;
    state.trace = v.trace || '';
    state.cols = v.cols || VIEW_DEFAULT.cols;
    var lo = document.getElementById('req-layout'); if (lo) lo.value = state.layout;
    var ts = document.getElementById('req-trace'); if (ts) ts.value = state.trace;
    var mc = document.getElementById('req-matrix-cols'); if (mc) mc.value = state.cols;
    setView(v.view || VIEW_DEFAULT.view);
    if (state.lastGraph && !noDraw) draw(state.lastGraph);
  }

  // Push a history entry only when the view actually changed.
  function pushUrl() {
    var cur = history.state || {};
    var s = snapshot();
    if (cur.focus === s.focus && cur.depth === s.depth && cur.config === s.config) return;
    history.pushState(s, '', urlFor());
  }

  function resetDetail() {
    state.selected = null;
    detailEl.textContent = '';
    var hint = document.createElement('p');
    hint.className = 'req-hint';
    hint.textContent = 'Select a node to see its element card.';
    detailEl.appendChild(hint);
  }

  function focusOn(id, push) {
    resetDetail();
    state.focus = id;
    document.getElementById('req-root').value = id;
    if (push) pushUrl();
    return load();
  }

  // Filter checkboxes: a ticked box restricts to that value; none ticked in a group restricts nothing.
  function readFilters() {
    var f = {};
    ['types', 'verification', 'asil'].forEach(function (g) {
      f[g] = Array.prototype.map.call(document.querySelectorAll('#req-filters input[data-group="' + g + '"]:checked'), function (i) { return i.value; });
    });
    state.filters = f;
  }

  function addFilterBox(host, group, value, label) {
    var lab = document.createElement('label');
    var cb = document.createElement('input');
    cb.type = 'checkbox'; cb.value = value; cb.setAttribute('data-group', group);
    cb.addEventListener('change', function () { readFilters(); if (state.lastGraph) draw(state.lastGraph); });
    lab.appendChild(cb);
    lab.appendChild(document.createTextNode(' ' + (label || value)));
    host.appendChild(lab);
  }

  function syncTypeFilters(raw) {
    var host = document.getElementById('req-filter-types');
    if (!host) return;
    var have = {};
    Array.prototype.forEach.call(host.querySelectorAll('input'), function (i) { have[i.value] = true; });
    raw.nodes.map(function (n) { return n.type; }).filter(Boolean).sort().forEach(function (t) {
      if (!have[t]) { have[t] = true; addFilterBox(host, 'types', t); }
    });
  }

  var searchSeq = 0;
  function initSearch() {
    var input = document.getElementById('req-search');
    var list = document.getElementById('req-search-results');
    if (!input || !list) return;
    var timer = null;
    input.addEventListener('input', function () {
      clearTimeout(timer);
      ++searchSeq; // anything already in flight is superseded, even if this input is empty
      timer = setTimeout(function () {
        var q = input.value.trim();
        var my = searchSeq;
        list.textContent = '';
        if (!q) return;
        var cfg = document.getElementById('req-config').value.trim();
        var url = '/api/req-graph/search?q=' + encodeURIComponent(q) + (cfg ? '&config=' + encodeURIComponent(cfg) : '');
        fetch(url).then(function (r) { return r.json().then(function (j) { return { ok: r.ok, body: j }; }); }).then(function (res) {
          if (my !== searchSeq) return; // a newer search superseded this one
          list.textContent = '';
          if (!res.ok) { var li = document.createElement('li'); li.textContent = (res.body && res.body.error) || 'Search failed.'; list.appendChild(li); return; }
          if (!res.body.results.length) { var none = document.createElement('li'); none.textContent = 'No matches.'; list.appendChild(none); return; }
          res.body.results.forEach(function (r) {
            var li = document.createElement('li');
            var b = document.createElement('button');
            b.type = 'button';
            b.textContent = r.id + (r.name ? ' — ' + r.name : '') + ' (' + (r.type || '?') + ')';
            b.addEventListener('click', function () { ++searchSeq; clearTimeout(timer); list.textContent = ''; input.value = ''; focusOn(r.id, true); });
            li.appendChild(b);
            list.appendChild(li);
          });
          if (res.body.truncated) { var more = document.createElement('li'); more.textContent = 'More matches — refine the search.'; list.appendChild(more); }
        }).catch(function () { if (my === searchSeq) list.textContent = 'Search failed.'; });
      }, 200);
    });
  }

  function init() {
    root = document.getElementById('req-explorer');
    if (!root) return;
    svg = document.getElementById('req-graph');
    statusEl = document.getElementById('req-status');
    detailEl = document.getElementById('req-detail');
    state.focus = root.getAttribute('data-focus') || '';
    state.depth = parseInt(root.getAttribute('data-depth') || '1', 10) || 1;
    state.config = root.getAttribute('data-config') || '';
    document.getElementById('req-depth').value = String(state.depth);
    var kinds = document.getElementById('req-kinds');
    EDGE_KINDS.forEach(function (k) {
      var lab = document.createElement('label');
      var cb = document.createElement('input');
      cb.type = 'checkbox'; cb.value = k; cb.checked = true;
      cb.addEventListener('change', load);
      lab.appendChild(cb);
      lab.appendChild(document.createTextNode(' ' + k));
      kinds.appendChild(lab);
    });
    document.getElementById('req-controls').addEventListener('submit', function (ev) {
      ev.preventDefault();
      resetDetail();
      state.focus = document.getElementById('req-root').value.trim();
      state.depth = parseInt(document.getElementById('req-depth').value, 10) || 1;
      state.config = document.getElementById('req-config').value.trim();
      pushUrl();
      load();
    });
    window.addEventListener('popstate', function (ev) {
      var s = ev.state || snapshot();
      state.focus = s.focus === undefined ? '' : s.focus;
      state.depth = s.depth || 1;
      state.config = s.config || '';
      document.getElementById('req-root').value = state.focus;
      document.getElementById('req-depth').value = String(state.depth);
      document.getElementById('req-config').value = state.config;
      resetDetail();
      // Fields an entry lacks (older entries) come from the address bar; load() redraws, so skip the stale draw.
      var fromUrl = parseViewState(location.search);
      var view = {};
      ['layout', 'view', 'trace', 'cols'].forEach(function (k) { view[k] = ev.state && ev.state[k] !== undefined ? ev.state[k] : fromUrl[k]; });
      applyView(view, true);
      load();
    });
    var fv = document.getElementById('req-filter-verification');
    if (fv) [['verified', 'verified'], ['planned', 'planned'], ['unverified', 'unverified'], ['na', 'other element']].forEach(function (p) { addFilterBox(fv, 'verification', p[0], p[1]); });
    var fa = document.getElementById('req-filter-asil');
    if (fa) ['QM', 'A', 'B', 'C', 'D'].forEach(function (a) { addFilterBox(fa, 'asil', a); });
    var vm = document.getElementById('req-view-matrix');
    if (vm) vm.addEventListener('click', function () { setView(state.view === 'matrix' ? 'graph' : 'matrix'); syncUrl(); });
    var mc = document.getElementById('req-matrix-cols');
    if (mc) mc.addEventListener('change', function () { state.cols = mc.value; if (state.drawn) renderMatrix(state.drawn); syncUrl(); });
    var vt = document.getElementById('req-view-table');
    if (vt) vt.addEventListener('click', function () { setView(state.view === 'table' ? 'graph' : 'table'); syncUrl(); });
    ['json', 'csv', 'svg'].forEach(function (k) {
      var b = document.getElementById('req-export-' + k);
      if (b) b.addEventListener('click', function () { exportAs(k); });
    });
    var lo = document.getElementById('req-layout');
    if (lo) lo.addEventListener('change', function () { state.layout = lo.value; if (state.lastGraph) draw(state.lastGraph); syncUrl(); });
    var ts = document.getElementById('req-trace');
    if (ts) ts.addEventListener('change', function () { state.trace = ts.value; if (state.lastGraph) draw(state.lastGraph); syncUrl(); });
    // The URL wins over any form state the browser restored; the initial entry then holds the full state.
    applyView(parseViewState(location.search));
    history.replaceState(snapshot(), '', location.href);
    initSearch();
    document.addEventListener('syscribe:reload', load);
    load();
  }

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = { parseViewState: parseViewState, viewStateQuery: viewStateQuery, vmodelColumn: vmodelColumn, matrixModel: matrixModel, tableRows: tableRows, csvCell: csvCell, toCsv: toCsv, exportName: exportName, CATEGORIES: CATEGORIES, layoutGraph: layoutGraph, tracePath: tracePath, categoryOf: categoryOf, jumpLinks: jumpLinks, effectiveAsil: effectiveAsil, filterGraph: filterGraph, EDGE_KINDS: EDGE_KINDS };
  } else if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init); else init();
  }
})();
