// Live-reload client (Phase 6, "/ws client wiring" — see the
// dreamy-whistling-treasure refactor plan).
//
// `ModelStore::commit` (crates/syscribe-server/src/state.rs) pushes
// {"event":"reload"} to the store's broadcast channel on every successful
// guarded write; `main.rs`'s file-watcher independently does the same for
// edits made outside the running server (e.g. hand-editing a model file).
// `GET /ws` (crates/syscribe-server/src/routes/ws.rs) forwards every
// broadcast verbatim to any connected client. This script is the client
// half: it opens that WebSocket, and on a reload event refreshes the
// sidebar tree and the currently open diagram tab (if any), discarding any
// cached/local state so the browser reflects the on-disk model exactly.
//
// Plain vanilla script, no build step, no dependencies — loaded as a plain
// <script src="..." defer> tag exactly like htmx.min.js/mermaid.min.js/
// diagram-editor.js (see base.html's <head>). Not part of the frontend/ TS
// build. Relies on globals defined by base.html's inline <script> (tabState,
// activeTabId, activateTab, refreshActiveTab) and by htmx.min.js/
// diagram-editor.js (htmx, window.DiagramEditor) — all of those load/run
// before this file because deferred scripts execute in document order and
// base.html's tab-management <script> block is a non-deferred inline
// script that runs during parsing, ahead of any deferred script.
(function () {
  'use strict';

  var RECONNECT_MIN_MS = 1000;
  var RECONNECT_MAX_MS = 10000;
  var reconnectDelay = RECONNECT_MIN_MS;
  var socket = null;

  function wsUrl() {
    var proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    return proto + '//' + window.location.host + '/ws';
  }

  function handleReload() {
    if (window.htmx) {
      htmx.ajax('GET', '/ui/tree', { target: '#tree-root', swap: 'innerHTML' });
    } else {
      console.error('[live-reload] htmx not available, cannot refresh tree');
    }

    if (typeof refreshActiveTab === 'function') {
      refreshActiveTab().catch(function (err) {
        console.error('[live-reload] failed to refresh active diagram tab', err);
      });
    }
  }

  function connect() {
    socket = new WebSocket(wsUrl());

    socket.addEventListener('open', function () {
      reconnectDelay = RECONNECT_MIN_MS;
    });

    socket.addEventListener('message', function (event) {
      var msg;
      try {
        msg = JSON.parse(event.data);
      } catch (err) {
        console.error('[live-reload] malformed message from /ws', event.data, err);
        return;
      }
      if (msg && msg.event === 'reload') {
        handleReload();
      }
    });

    socket.addEventListener('close', function () {
      socket = null;
      window.setTimeout(connect, reconnectDelay);
      reconnectDelay = Math.min(reconnectDelay * 2, RECONNECT_MAX_MS);
    });

    socket.addEventListener('error', function () {
      // The browser fires 'close' right after 'error' on a failed/dropped
      // connection, so the close handler above owns all reconnect/backoff
      // logic; just make sure the socket is torn down.
      if (socket) {
        socket.close();
      }
    });
  }

  connect();
})();
