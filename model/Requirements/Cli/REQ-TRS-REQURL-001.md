---
type: Requirement
id: REQ-TRS-REQURL-001
name: "The Requirements Explorer keeps its view state in the URL"
status: draft
reqDomain: software
reqClass: system
tags:
  - server
---

`syscribe-server`'s Requirements Explorer shall carry its view state in the page URL so a link, reload or Back restores it (GH #269, phase 8).

## Behavior

- Besides `focus`, `depth` and `config`, the URL query carries `layout` (`hops` | `v-model`), `view` (`graph` | `table` | `matrix`), `trace` (`tests` | `architecture` | `features` | `safety` | `security`) and `cols` (the matrix column category, same vocabulary as `trace`). Defaults (`hops`, `graph`, no trace, `tests`) are omitted from the URL.
- Parsing is a pure function of the query string: a value outside its vocabulary is ignored (the default applies), never an error and never reflected unescaped into the page. Building the query is the inverse for every valid state.
- The page applies the parsed state to its controls on load (the URL wins over any form state the browser restored) and on Back/Forward. Changing one of these view controls updates the current history entry in place, so Back does not step through every toggle; focus, depth and configuration navigation keeps the entry rules of REQ-TRS-REQEXPL-001 and carries the current view state along.
- The type, verification and ASIL filters, the relation checkboxes and the selected node are not part of the URL and are not restored on reload or Back. A URL fragment is kept when the view state is rewritten.
