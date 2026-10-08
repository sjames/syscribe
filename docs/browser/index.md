# Web Browser

`BROWSER · OVERVIEW`

The web server is a single Rust binary (`syscribe-server`) that parses a model directory, builds an in-memory element graph, and serves a browser-based UI with live file-system watching.

## Starting the server

```bash
cargo run --package syscribe-server -- -m model/
# INFO  Loaded <N> elements
#
#   Model browser: http://0.0.0.0:3000/
```

Pass the model root with `-m <path>` (or set `SYSCRIBE_MODEL`); change the listen address with `--bind <addr:port>` (default `0.0.0.0:3000`). The server watches the directory for changes and pushes a reload event to connected clients over WebSocket.

`syscribe-server` is built from source (`cargo build --workspace` or `cargo install --path crates/syscribe-server`); it is not shipped as a prebuilt release binary. For a static, server-free view of a model, use `syscribe -m model/ export-html <out-dir>` instead.

## Stack

| Layer | Technology |
|---|---|
| HTTP server | Axum (Rust) |
| HTML templates | Askama (server-side rendering) |
| Dynamic updates | HTMX (partial-page swaps — no JS framework) |
| Diagram rendering | Mermaid.js for `diagramKind: Mermaid`; an editable sprotty-based diagram client with ELK layout (`sprotty-elk` + `elkjs`) for every other kind |
| File watching | `notify` crate + WebSocket push |

All JavaScript (HTMX, Mermaid, the bundled diagram editor including `sprotty-elk` and `elkjs`) is vendored and served from the binary under `/static/` — no CDN is needed at runtime and no Node runtime is needed on the server.

## UI routes

| Path | Description |
|---|---|
| `GET /` | Root — renders the model tree browser |
| `GET /ui/tree?parent=<qname>` | HTMX — returns tree items for a namespace (top level when `parent` is omitted) |
| `GET /ui/detail/<qname>` | HTMX — element detail panel (rendered Markdown, custom fields, generated member list, edit form) |
| `GET /ui/diagram/<qname>` | HTMX — diagram panel (Mermaid, or the host for the sprotty editor) |
| `GET /planning` | The live planning dashboard page |
| `GET /ui/planning/board?who=&done=` | HTMX — the board fragment the dashboard re-fetches |
| `GET /ui/element-card/<qname>` | HTMX — the read-only card in the diagram editor's side panel: identity and the rendered Markdown body; an inline feature resolves to its owner or type |
| `GET /static/<path>` | Vendored JS/CSS assets |

## API routes

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/elements` | List all elements; optional `?type=PartDef` filter |
| `POST` | `/api/elements` | Create an element (guarded write) |
| `GET` | `/api/elements/<qname>` | Single element JSON |
| `PUT` | `/api/elements/<qname>` | Update an element's frontmatter fields and/or body (guarded write) |
| `DELETE` | `/api/elements/<qname>` | Delete an element; refused while other elements still reference it unless `?force=true` (guarded write) |
| `GET` | `/api/children?of=<qname>` | Direct children of an element |
| `GET` | `/api/connections?of=<qname>` | An element's connection frontmatter (`connections`, flow/binding/succession connections, `exhibitsStates`) |
| `POST` | `/api/connections` | Add a `connections:` entry to the element named by `qname` in the body (guarded write) |
| `DELETE` | `/api/connections` | Remove a `connections:` entry (guarded write) |
| `GET` | `/api/diagrams/model/<qname>` | A `Diagram` element as a nested sprotty graph model (from its manifest or derived from its `subject:`), with ELK `layoutOptions`, the `pinned` set and the resolved style per element; `404` for Mermaid, non-diagram or unknown names |
| `PATCH` | `/api/diagrams/layout/<qname>` | Persist pins: `{ "<shapeId>": {x, y, w?, h?} }` writes the pin (`w`/`h` when given); a `null` value removes that shape's pin (guarded write) |
| `DELETE` | `/api/diagrams/layout/<qname>` | Remove every pin — drops the diagram's whole `layout:` key (the *Auto-layout* button; guarded write) |
| `PUT` | `/api/diagrams/svg/<qname>` | Save a companion SVG: body `{ "svg": "<svg …>…</svg>" }` is written to `svgFile:` (default `<stem>.svg` beside the `.md`), setting `svgMode: companion`/`svgFile:` and appending an `<img>` to the body when absent (guarded write; refused unless the body is an SVG document) |
| `POST` | `/api/diagrams/shapes/<qname>` | Add an existing element to a manifest diagram as a shape (unpinned unless `x`/`y` are given; refused for a derived diagram; guarded write) |
| `GET` | `/api/validation` | Validation findings JSON (includes `qname` per finding) |
| `WS` | `/ws` | Live model-change events |

## Editing through the browser

The web service is not read-only: the `POST`/`PUT`/`DELETE`/`PATCH` routes above write model files. Every one of them goes through the same guarded-write engine the MCP write tools use — the change is applied to a candidate copy of the model and re-validated before anything touches disk; a create or update that would leave a reference unresolved is refused, and a delete is refused while other elements still reference the target (unless forced). Each write returns HTTP 200 with a JSON body carrying `written`, the validation delta (`newErrors`, `resolvedErrors`, `newWarnings`, `resolvedWarnings`), a unified `diff`, and a `reason` when refused. Pass `"dryRun": true` in the body (or `?dryRun=true` on `DELETE /api/elements`) to preview without writing. A committed write reloads the model and notifies every connected client over `/ws`.

The detail panel's **Edit** button edits an element's name, documentation and remaining frontmatter (as YAML); diagrams that are not Mermaid open in an editable diagram view where shapes can be created, deleted, connected and moved.

Elements synthesized from a multi-element sheet (FMEA/TARA rows, `FeatureModel` `featureTree:` entries) are refused by the update and delete routes — edit the sheet instead.

## Element detail panel

Clicking a tree item opens the detail panel, which shows:

1. **Element metadata** — name, type badge, qualified name, and stable ID (for `Requirement`, `TestCase`, `ADR` and every other id-identified type) in a monospace chip
2. **Documentation** — full rendered Markdown, including tables, code blocks, and embedded Mermaid diagrams
3. **Custom fields** — any `customFields` as a read-only key/value table
4. **Members** — for a package (or any element that owns children), its direct members, generated from the directory tree
5. **View source** — a link to the element's hosted source file, when `[links]` is configured in `.syscribe.toml`

Validation findings are available as JSON from `/api/validation`; run `syscribe validate` for the full report.

!!! note "Locale documentation variants"
    §3.10 locale variants (files with `locale:` + `qualifiedName:`) are attached to their element by the loader and printed by `syscribe show` as one `Documentation (<locale>)` section each. The web UI detail panel does not show them yet — it renders only the element's own documentation body.

## Diagram rendering

A `diagramKind: Mermaid` diagram is still rendered client-side by Mermaid.js from the ` ```mermaid ` block in its body. Every other kind is served as a Diagram IR (`syscribe_model::vis`): the server builds one graph per `Diagram` element — from its `shapes:`/`edges:`/`layout:` manifest (the single parser that also reports a malformed manifest as `E405`), or derived from its `subject:` when it declares no `shapes:` — and `GET /api/diagrams/model/<qname>` hands it to the sprotty editor as a nested graph model: ports inside their block, blocks inside the boundary, a label and compartments per node, edges at the root, the ELK `layoutOptions` for the diagram's kind, the `pinned` set, and the resolved style (colours, arrowheads, stereotype banners, port glyphs) per element, owned once in Rust (`vis::style`) so the views hold no colour table. The browser lays the graph out; the server never computes positions and there is no server-side SVG renderer.

## Creating a diagram

The **+ Diagram** button in the model browser's header opens the New diagram dialog (`REQ-TRS-VIS-023`). Give it a **name** (letters, digits and underscores; no spaces or hyphens), a **kind** (BDD, IBD, state machine, action flow, sequence, requirement tree or allocation map), and choose how to **start**:

- **Derive from a subject** writes a two-line diagram — `diagramKind:` and `subject:` — that follows the model from then on. The subject box suggests only the element types that kind accepts (a part or item for an IBD, an action or use case for a sequence diagram, a package or requirement for a requirement tree), and anything else is refused before a request is sent.
- **Blank** writes an empty manifest (`shapes: {}`) for you to fill with **+**, **↔** and the pin buttons; a subject is optional and, when given, owns the connections you draw.

The **package** defaults to a package named `Diagrams` when the model has one, else the model root; the list offers every package element. Creating goes through the same guarded write as every other edit, then closes the dialog and opens the diagram in a tab. A refusal (a name already in use, a rule the model engine enforces) appears in red inside the dialog and writes nothing; a warning the new diagram raises, such as `W401` for a subject that resolves to nothing, is shown after it is created.

## Planning dashboard

**Planning** in the header opens `/planning` (`REQ-TRS-VIS-027`), a live board of the model's `PlanningItem`s:

- **Summary** counts per status.
- **Working now** groups every `in_progress` or claimed item by who is on it. An agent that holds a claim (`syscribe claim <PI> --by <agent>`) shows a pulsing dot and how long ago it claimed (`claimedAt:`); an `assignedTo:` person shows by the display name from `[users]`.
- **Board** has a column per status, each card showing id, type, parent, assignee, claimant and what it is `blockedBy:`. A card opens its detail dialog.
- **Who** and **show done** filter the board (done items are hidden but counted); a name in *Working now* is a filter link.

It refreshes with no reload: on every model-reload event of the same WebSocket that updates the tree (so `claim`, `release`, status edits and an agent's writes appear within moments), on a 15 s timer, and claim ages tick every second. The **live** badge turns red when the socket is down. Read-only; change items with the CLI, MCP or the detail dialog.

## Reading an element from a diagram

Click a shape, port or edge and an **Element** panel opens over the diagram's right-hand edge (`REQ-TRS-VIS-026`) with the depicted element's identity — name, type, qualified name, id, status — and the **body of its Markdown file rendered** (headings, lists, tables, code, Mermaid blocks), from `GET /ui/element-card/<qname>`. It follows the selection; selecting several shapes or using the connect tool leaves it as it is, and **✕** closes it until the next click. **Open full detail** opens the element's dialog for editing.

A port, part usage or attribute drawn on a diagram is usually an inline feature of its owner, not a file of its own. The panel then names it ("Feature `pdu` of `UAV::Power::PowerSystem`"), lists what it declares (`typedBy`, `direction`, `multiplicity`, `unit`) and shows the documentation of its type when it has one, else its owner's. A reference that names nothing says so.

## The diagram editor

Opening a non-Mermaid diagram runs **ELK automatic layout in the browser**: every node, port, compartment and label arrives with its size already computed by the server from shared text metrics (`REQ-TRS-VIS-017`; the client measures in the DOM only what came without one), then `sprotty-elk` lays the sized graph out with the bundled `elkjs` (layered, top-down for a BDD with supertypes above subtypes, left-to-right with nesting for an IBD, orthogonal edge routing, ports on the block border, edge keywords and labels placed along the route). Nothing is written back: an automatic layout lives only in the browser until you act, so `git diff` on a diagram file always shows a human's intent.

Entries in `layout:` are **pins**. A pinned shape keeps the position (and size, when `w`/`h` are recorded) its entry holds while ELK places the unpinned rest around it; when every shape is pinned ELK takes the positions as given and only routes the edges. A pin that names no shape is `W416`.

Three gestures and three buttons change the model, each one guarded write that returns the usual `WriteResponse` delta and shows a toast on refusal:

- **⊕ Add existing element** opens a picker over the whole model (type to filter by name or qualified name, prefix matches first; diagrams are not offered). Choosing one `POST`s it to `/api/diagrams/shapes/<qname>`, which lists it under `shapes:` with the kind of the element's type and **no pin**, so ELK places it around the pinned ones; the element's own file is never touched. An element already on the diagram, an unresolved name and a **derived** diagram are refused in the picker with the reason (a derived diagram follows its subject; narrow or widen it with `include:`/`exclude:`, or start a blank one with **+ Diagram**). Relationships to shapes already on the diagram are not added for you; use **↔** Connect.
- **Drag** a shape to pin it — one `PATCH /api/diagrams/layout/<qname>` carrying that shape's new position (relative to its parent). A refused move snaps back.
- **Pin all** writes every placed shape's current position and ELK-sized width/height as pins in a single `PATCH`, so the picture ELK produced becomes explicit, reviewable `layout:` entries and reopens exactly as it looks now.
- **Auto-layout** clears every pin with `DELETE /api/diagrams/layout/<qname>` (the whole `layout:` key goes), re-fetches the diagram and lets ELK lay it out from scratch. The cached picture is only dropped once the delete was accepted.
- **Save companion SVG** serialises the current render — with `sysml:ref` on every node and port, `sysml:source`/`sysml:target` on every edge and sprotty's interactive state stripped — and `PUT`s it to `/api/diagrams/svg/<qname>`. The server writes it to the diagram's `svgFile:` (default `<stem>.svg` beside the `.md`), sets `svgMode: companion` and `svgFile:` when absent, and appends one `<img>` to the body (never twice), so GitHub and `export-html` show the picture you approved. Saving does not write pins; press *Pin all* for that. A body that is not an SVG document is refused and nothing is written.
- **Add** and **Delete** create and remove elements as before (`REQ-TRS-DE-004`); on a derived diagram the element is created under the subject and the view regenerates on reload.
- **Connect** mode is port-aware: click a source port, then a target port. An `out` port joins an `in` port (an `inout` or undirected port joins anything); two ports of the same direction are refused. You may click a block instead of a port when the block has exactly one port compatible with the other end — a block with no ports, no compatible pair, or several compatible pairs is refused with a toast naming the reason (for several pairs, the toast lists them and asks you to connect the two ports directly). An accepted gesture adds a `connections:` entry on the diagram's `subject:` with `from`/`to` spelled as dotted feature chains relative to it (`battery.powerOut` → `pdu.powerIn`), plus the manifest edge on a manifest diagram.

The ELK bundle is vendored (`sprotty-elk` and `elkjs` in `crates/syscribe-server/frontend/package.json`, built by esbuild into `static/js/diagram-editor.js`); `npm test` in that directory runs the bundled ELK over a fixture IBD and checks that ports sit on their block's border and siblings do not overlap.

After any committed write the file watcher reloads the element and notifies all connected clients.
