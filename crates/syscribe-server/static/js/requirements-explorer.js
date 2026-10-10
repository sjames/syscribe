// Requirements Explorer client (GH #269, REQ-TRS-REQEXPL-001). Plain vanilla script, no build step.
// Fetches /api/req-graph, lays the neighbourhood out in layers by hop distance and draws it as SVG.
// All model text is written with textContent / attributes — never innerHTML — except the server's own
// element-card fragment, which the server renders and escapes.
(function () {
  'use strict';

  var NODE_W = 190, NODE_H = 44, GAP_X = 96, GAP_Y = 22, PAD = 24;
  var EDGE_KINDS = ['derivedFrom', 'satisfies', 'verifies', 'allocatedTo', 'refines', 'supersedes',
    'derivedFromSafetyGoal', 'breakdownAdr', 'blockedBy', 'covers', 'analyses', 'runsOn', 'achieves', 'evidence', 'appliesWhen'];
  var FILL = { verified: '#e3f4e5', planned: '#fff3d6', unverified: '#fde4e1', na: '#eceff4' };
  var STROKE = { verified: '#2e7d32', planned: '#b7791f', unverified: '#c0392b', na: '#7a869a' };

  // Pure layout: layers by hop distance from the root over the undirected edges. Nodes unreachable from
  // the root (the API never returns any) go one column past the last. Deterministic: ids sorted per column.
  function layoutGraph(g) {
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
    return { nodes: nodes, edges: g.edges || [], width: width, height: height };
  }

  function el(name, attrs, text) {
    var e = document.createElementNS('http://www.w3.org/2000/svg', name);
    Object.keys(attrs || {}).forEach(function (k) { e.setAttribute(k, attrs[k]); });
    if (text !== undefined) e.textContent = text;
    return e;
  }

  function truncate(s, n) { s = s || ''; return s.length > n ? s.slice(0, n - 1) + '…' : s; }

  var state = { focus: '', depth: 1, config: '', kinds: null, selected: null, lastGraph: null };
  var root, svg, statusEl, detailEl;

  function setStatus(t) { if (statusEl) statusEl.textContent = t || ''; }

  function draw(graph) {
    state.lastGraph = graph;
    var lay = layoutGraph(graph);
    while (svg.firstChild) svg.removeChild(svg.firstChild);
    svg.setAttribute('width', lay.width);
    svg.setAttribute('height', lay.height);
    svg.setAttribute('viewBox', '0 0 ' + lay.width + ' ' + lay.height);
    var defs = el('defs');
    var marker = el('marker', { id: 'rg-arrow', viewBox: '0 0 10 10', refX: '9', refY: '5', markerWidth: '7', markerHeight: '7', orient: 'auto-start-reverse' });
    marker.appendChild(el('path', { d: 'M0,0 L10,5 L0,10 z', fill: '#7a869a' }));
    defs.appendChild(marker);
    svg.appendChild(defs);
    var pos = {};
    lay.nodes.forEach(function (n) { pos[n.id] = n; });
    var labels = [];
    var seenPair = {};
    lay.edges.forEach(function (e) {
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
      var path = el('path', { d: d, class: 'rg-edge', 'marker-end': 'url(#rg-arrow)', 'data-kind': e.kind });
      path.appendChild(el('title', {}, e.from + ' → ' + e.to + ' (' + e.kind + ')'));
      svg.appendChild(path);
      var pk = [e.from, e.to].sort().join('|');
      var k = seenPair[pk] = (seenPair[pk] || 0) + 1;
      labels.push(el('text', { x: (x1 + x2) / 2 + (a.col === b.col ? 44 : 0), y: (y1 + y2) / 2 - 4 + (k - 1) * 12, 'text-anchor': 'middle', class: 'rg-edge-label' }, e.kind));
    });
    lay.nodes.forEach(function (n) {
      var v = n.verification || 'na';
      var vlabel = { verified: 'verified', planned: 'verification planned', unverified: 'not verified', na: '' }[v] || '';
      var summary = (n.type || '') + ' ' + n.id + (n.name ? ' — ' + n.name : '') + (n.status ? ' [' + n.status + ']' : '') + (vlabel ? ', ' + vlabel : '');
      var g = el('g', { class: 'rg-node' + (n.root ? ' root' : '') + (state.selected === n.id ? ' selected' : ''), 'data-id': n.id, tabindex: '0', role: 'button', 'aria-label': summary, 'aria-pressed': state.selected === n.id ? 'true' : 'false' });
      g.appendChild(el('title', {}, summary));
      g.appendChild(el('rect', { x: n.x, y: n.y, width: n.w, height: n.h, rx: 6, fill: FILL[v] || FILL.na, stroke: STROKE[v] || STROKE.na }));
      g.appendChild(el('text', { x: n.x + 8, y: n.y + 18 }, truncate(n.id, 28)));
      var sub = [n.type, n.asil ? 'ASIL ' + n.asil : null, n.status, v === 'verified' ? '✓' : v === 'unverified' ? '✗' : v === 'planned' ? '◐' : null].filter(Boolean).join(' · ');
      g.appendChild(el('text', { x: n.x + 8, y: n.y + 34, class: 'rg-sub' }, truncate(sub, 34)));
      g.addEventListener('click', function () { select(n); });
      g.addEventListener('keydown', function (ev) { if (ev.key === 'Enter' || ev.key === ' ') { ev.preventDefault(); select(n); } });
      svg.appendChild(g);
    });
    labels.forEach(function (t) { svg.appendChild(t); }); // above the nodes, with a halo (see the stylesheet)
    setStatus(graph.nodes.length + ' element(s), ' + (graph.edges || []).length + ' relation(s)' + (graph.truncated ? ' — truncated at the server node limit — lower the depth or untick relations' : ''));
  }

  function qnamePath(q) { return q.split('::').map(encodeURIComponent).join('/'); }

  function select(n) {
    state.selected = n.id;
    Array.prototype.forEach.call(svg.querySelectorAll('.rg-node'), function (g) {
      var on = g.getAttribute('data-id') === n.id;
      g.classList.toggle('selected', on);
      g.setAttribute('aria-pressed', on ? 'true' : 'false');
    });
    detailEl.textContent = '';
    var bar = document.createElement('p');
    var btn = document.createElement('button');
    btn.type = 'button';
    btn.textContent = 'Focus here';
    btn.addEventListener('click', function () { focusOn(n.id, true); });
    bar.appendChild(btn);
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

  function clearGraph() { while (svg.firstChild) svg.removeChild(svg.firstChild); state.lastGraph = null; }

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
    return '/requirements?' + p.toString();
  }

  function snapshot() { return { focus: state.focus, depth: state.depth, config: state.config }; }

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
    // The initial entry gets the seeded state, so Back to it restores focus, depth and configuration.
    history.replaceState(snapshot(), '', location.href);
    window.addEventListener('popstate', function (ev) {
      var s = ev.state || snapshot();
      state.focus = s.focus === undefined ? '' : s.focus;
      state.depth = s.depth || 1;
      state.config = s.config || '';
      document.getElementById('req-root').value = state.focus;
      document.getElementById('req-depth').value = String(state.depth);
      document.getElementById('req-config').value = state.config;
      resetDetail();
      load();
    });
    document.addEventListener('syscribe:reload', load);
    load();
  }

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = { layoutGraph: layoutGraph, EDGE_KINDS: EDGE_KINDS };
  } else if (typeof document !== 'undefined') {
    if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init); else init();
  }
})();
