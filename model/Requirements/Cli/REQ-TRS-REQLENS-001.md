---
type: Requirement
id: REQ-TRS-REQLENS-001
name: "The Requirements Explorer offers a table lens and exports the drawn graph"
status: draft
reqDomain: software
reqClass: system
tags:
  - server
---

`syscribe-server`'s Requirements Explorer shall present the drawn graph as an accessible table and export it as JSON, CSV and SVG (GH #269, phase 5; the V-model and matrix lenses remain).

## Behavior

- A "Table" view toggle shows the drawn (filtered) graph as two HTML tables — elements (id, type, name, status, ASIL, verification, hop distance from the root) and relations (from, to, kind) — instead of the SVG (the toggle keeps one label and exposes its state with `aria-pressed`; table headers carry `scope`). After a failed or empty load nothing is exported or tabulated from the previous graph. Every id cell is a button that selects the element, so the table is a keyboard-and-screen-reader equivalent of the graph. The trace highlight marks rows on the trace.
- Export produces, from the drawn graph and never from a fresh request: JSON (the filtered API response with its `hidden` count), CSV (one table, `record,id,type,name,status,asil,verification,hop,from,to,kind`, a row per element and per relation, CRLF line ends and a UTF-8 byte-order mark for spreadsheets) and the SVG as displayed, made standalone with its styles inlined and a white background. Files are generated in the browser and named `req-graph-<root>.<ext>` with the root reduced to `[A-Za-z0-9._-]`.
- CSV cells are RFC 4180 quoted, and a cell that begins with `=`, `+`, `-`, `@`, tab or carriage return is prefixed with a single quote so a spreadsheet cannot evaluate model text as a formula. The conversion functions are pure functions of the graph.
