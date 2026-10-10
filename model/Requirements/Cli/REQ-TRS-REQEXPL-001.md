---
type: Requirement
id: REQ-TRS-REQEXPL-001
name: "The server has a Requirements Explorer page that shows and navigates the traceability graph"
status: draft
reqDomain: software
reqClass: system
tags:
  - server
---

`syscribe-server` shall serve a Requirements Explorer page over the graph API of REQ-TRS-REQGRAPH-001 (GH #269, phase 2: the page; filters beyond edge kinds, alternate lenses and export follow).

## Behavior

- `GET /requirements[?focus=<id|qname>][&depth=N][&config=C]` serves a page with the explorer: a controls bar (root, depth 1–6, edge-kind checkboxes, configuration), an SVG graph and a detail pane. `focus`, `depth` and `config` seed the initial state as HTML-escaped data attributes (a hostile value cannot inject markup); a missing or out-of-range `depth` falls back to 1/6.
- The graph is drawn in layers by hop distance from the root, nodes coloured by the `verification` overlay (verified, planned, unverified, n/a) with the root emphasised, edges labelled by kind with arrowheads. Selecting a node shows its element card in the detail pane and a "Focus here" action recentres the graph on it; focusing pushes a history entry so back/forward and deep links (`?focus=`) work. The page reloads the graph on the server's live-reload event.
- The top navigation of every page links to the explorer. The client script is `/static/js/requirements-explorer.js`; its layout function is a pure function of the API response.
