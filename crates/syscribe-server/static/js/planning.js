// Planning dashboard client (`REQ-TRS-VIS-027`): re-fetches the board fragment
// on every model-reload event and on a timer, keeping the filters, and keeps
// the claim ages ("3m") ticking between fetches. The decisions that can be
// tested without a browser live in `planning-core.js`.
(function () {
  'use strict';
  var core = window.PlanningCore;
  var board = document.getElementById('planning-board');
  var who = document.getElementById('pp-who');
  var done = document.getElementById('pp-done');
  var conn = document.getElementById('pp-live');
  if (!board || !core) { return; }

  var params = new URLSearchParams(window.location.search);
  done.checked = params.get('done') === '1';
  var wantWho = params.get('who') || '';
  var inflight = 0;

  function url() { return core.boardUrl(who.value || wantWho, done.checked); }

  function refresh() {
    var mine = ++inflight;
    return fetch(url()).then(function (r) { if (!r.ok) { throw new Error(r.status); } return r.text(); }).then(function (html) {
      if (mine !== inflight) { return; }
      board.innerHTML = html;
      if (window.htmx) { window.htmx.process(board); }
      syncPeople();
      tickAges();
    }).catch(function () { /* the next event or timer retries */ });
  }

  // The people list lives in the fragment's filter links; mirror it into the select.
  function syncPeople() {
    var names = Array.prototype.map.call(board.querySelectorAll('.pb-filter'), function (a) { return a.dataset.who; });
    var current = who.value || wantWho;
    var all = core.mergePeople(Array.prototype.map.call(who.options, function (o) { return o.value; }), names, current);
    who.replaceChildren.apply(who, all.map(function (n) {
      var o = document.createElement('option');
      o.value = n; o.textContent = n === '' ? 'everyone' : n; return o;
    }));
    who.value = current;
  }

  function tickAges() {
    var now = Date.now();
    board.querySelectorAll('time[data-since]').forEach(function (t) {
      t.textContent = core.age(t.dataset.since, now);
    });
  }

  who.addEventListener('change', function () { wantWho = ''; refresh(); });
  done.addEventListener('change', refresh);
  board.addEventListener('click', function (ev) {
    var f = ev.target.closest('.pb-filter');
    if (f) { ev.preventDefault(); ev.stopPropagation(); wantWho = ''; who.value = f.dataset.who; refresh(); return; }
    if (ev.target.closest('.pb-showdone')) { ev.preventDefault(); done.checked = true; refresh(); }
  });

  // Live: the shared live-reload client announces every model reload.
  document.addEventListener('syscribe:reload', refresh);
  window.setInterval(refresh, 15000);
  window.setInterval(tickAges, 1000);
  window.addEventListener('syscribe:socket', function (e) {
    conn.classList.toggle('off', !e.detail.open);
    conn.textContent = e.detail.open ? 'live' : 'offline';
  });
  refresh();
})();
