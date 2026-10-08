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
| `GET /features` | The feature model viewer |
| `GET /api/feature-model/diagram?root=` | The feature diagram as a sprotty graph (optionally one feature's subtree) |
| `GET /api/feature-model/analysis` | Void, dead, core and false-optional features with the constraints responsible |
| `POST /api/feature-model/configure` | Propagate a partial selection: state of every feature, conflict with its cause, product count, one completed product |
| `GET /api/feature-model/configurations` | The stored `Configuration`s with their selections |
| `POST /api/feature-model/edit` | One semantic edit (`add`, `remove`, `rename`, `setGroup`, `setMandatory`, `move`, `addConstraint`, `removeConstraint`), with the validity delta; `preview` and `acceptWorse` options; returns the undo operation |
| `GET /api/feature-model/impact?feature=` | What a feature gates, who selects it, what depends on it |
| `GET /api/feature-model/export?format=svg\\|plantuml\\|mermaid` | The feature diagram as a download |
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

## Feature model viewer

**Features** in the header opens `/features` (`REQ-TRS-FMED-001`, `-002`), the product line's feature model as a **feature diagram** with the SAT analysis laid over it. It reads the model through the same engine as `feature-check --deep`, so the picture, the CLI and the MCP server cannot disagree.

- **Notation.** Each `FeatureDef` is a box. A filled circle above it means it is a *mandatory* member of its parent, a hollow one *optional*. Where a feature's `groupKind:` is `alternative` a hollow wedge joins its children (exactly one), where it is `or` a filled wedge (at least one). A dashed blue curve with an arrow is a `requires:` constraint, a dashed red one with arrows at both ends `excludes:`. A dashed outline is an abstract feature. Parameters and cardinality show inside the box.
- **Analysis.** Core features (in every product) are blue; **dead** features (in no product) are red, dashed and struck through; **false-optional** ones (declared optional but forced) amber with a double outline. The colour is a second channel: the outline differs too. A banner names the conflicting constraints when the model is **void**, and the side panel counts features, core, dead, false-optional and invalid configurations.
- **Inspector.** Click a feature to see its state and, for a dead or false-optional one, **why**, as the constraints responsible; what it requires, what requires it and what it excludes; and its Markdown documentation.
- **Collapse, search, fit.** The **−/+** at a feature's bottom-right collapses its subtree and counts what it hid. **Collapse all** shows the roots, **Expand all** everything, **Fit** the whole model. Typing in the search box finds features by name, id or qualified name, expands the path to each, highlights them and centres the first; Enter steps to the next. A model of more than 60 features opens collapsed to two levels.
- **Live.** The page refreshes on every model reload, so an edit by you, the CLI or an agent shows at once; the **live** badge turns red when the connection drops.
- **Export.** SVG, PlantUML and Mermaid downloads of the diagram, from the same writers as `syscribe diagram export`. A `Diagram` with `diagramKind: FeatureModel` and a `subject:` (a feature, a package of features or a feature-model sheet) puts the same diagram in the model; **+ Diagram** offers it.

### Configurator

**Configure** (`REQ-TRS-FMED-003`) turns the diagram into a configurator. Click a feature to **select** it, again to **deselect** it, again to leave it **open**. After every click the server propagates the choices through the whole model, and each feature is drawn as one of:

- a **solid green tick** or **solid red cross** badge: your choice;
- a **ring** with a tick or cross: **implied**, forced on or off by your choices and the model (a mandatory child, a `requires:`, an `excludes:`, a full alternative group);
- no badge: still **open**.

The panel shows how many valid products remain (`18 valid products`, or `at least 10,000` when counting stops at its budget) and how many features are chosen, implied and open. A click that no valid product can satisfy is **refused** and explained: which of your choices clash and the constraints they clash with, for example *No valid product has selecting Hex and selecting Quad (group 'Features::Propulsion' allows at most 1 selected child(ren))*. Your earlier choices stay as they were.

**Start from** loads a stored `Configuration` as the choices (a partial or invalid one is shown as such), **Clear** starts again. **Save** writes a new `Configuration` through a guarded write: the choices completed to one whole product the solver found, as `features:` with every feature true or false, `featureModel:` set to the package the features live in and a generated `CONF-GEN-nnn` id. A feature with a required `parameters:` entry then raises `W017` until you bind it. `POST /api/feature-model/configure` is the same computation for scripts.

### Editing

**Edit** (`REQ-TRS-FMED-004`) lets you change the feature model without leaving the diagram. Select a feature and the panel offers:

- An **abstract** checkbox (`setAbstract`): a grouping feature with no implementation of its own, drawn dashed and italic. It does not distinguish products (the configurator's product count is over concrete features, and an abstract feature is not counted among the choices still open), a configuration must not name it (`E238`; Save leaves abstract features out), and its value in a configuration is derived from the concrete selection; see the variability guide;
- **Rename**, and **Membership** (mandatory or optional) and **Children are** (*free*, *XOR* for one of, *OR* for any of);
- **Add child**, or **Add a root feature** when nothing is selected;
- its declared **constraints** with a **✕** to remove each, and a form to add a `requires` or `excludes` towards another feature;
- **Move under** another feature, or make it a root, and **Remove** (with everything below it, after a confirmation). You can also **drag** a feature onto another to make it that feature's child; dropped anywhere else it snaps back.

Each change is one semantic operation (`POST /api/feature-model/edit`) applied to the files the feature lives in, through the guarded-write engine: a feature is a file under its parent (a group is a directory with an `_index.md`), a new one gets a generated `FEAT-` id, a rename or move rewrites every reference to it including the keys of every `Configuration`, and removing a feature takes the `requires:`/`excludes:` entries and configuration choices that named it. **Undo** and **Redo** (Ctrl+Z, Ctrl+Shift+Z) reverse any change, as a stack for the session.

Before anything is written the server compares the SAT analysis of the model with and without the change. An edit that leaves things as they were, or better, is applied at once, and a toast says what improved. One that **makes the model worse** is held, and a dialog lists exactly how: *Electric becomes dead*, *the feature model becomes void* with the conflict, *Radio becomes false-optional*, *CONF-ONE-001 is no longer a valid product*. **Apply anyway** writes it; **Cancel** leaves the model untouched. Send `"preview": true` to the endpoint to ask without writing.

**Parameters** are listed with ✎ to edit and ✕ to remove each, and a form to add or replace one (name, type, range, default, required). Editing one keeps the keys the form does not show (`enumValues:`, `bindingTime:`). A parameter that a `Configuration` binds **cannot be removed**: the refusal names the configurations, and the Impact section lists each binding with a **✕** (`removeBinding`) in Edit mode. Remove the bindings first, then the parameter.

Features defined as entries of a single-file `featureTree:` sheet take every edit. A rename or move changes the entry's dotted path and those of the entries below it, writes down a derived id before it would change, and follows the dotted paths that name it in the sheet's `requires:`/`excludes:` and `crossTreeConstraints:`; a constraint is added inline on its entry and removed from the cross-tree list too; a removed feature takes its entries, its cross-tree constraints and the configuration choices and bindings that named it. A new child of a sheet entry is a file beside the sheet (the layouts may be mixed). A sheet entry can only be moved within its sheet, and not while a feature below it lives in a file; both are refused with the reason. The first edit rewrites the sheet's YAML in block style.

The same operations are the MCP tool `edit_feature` (`edit`, `dry_run`, `accept_worse`), with the same validity delta, the same hold for an edit that makes the model worse and an `undo` in the reply.

### Impact

Select a feature and the **Impact** section (`REQ-TRS-FMED-005`, `GET /api/feature-model/impact`) says what it affects: how many elements have an `appliesWhen:` that names it, grouped by type and listed with links into the model; how many more are conditioned only through a package that names it; which `Configuration`s select it and which deselect it; which features require or exclude it, and how many features hang below it. **What would removing it change?** previews the removal exactly as Edit would, without writing: the features that would become dead or false-optional, a model that would become void, the configurations that would stop being valid products.

### Matrix and comparison

**Matrix** (`REQ-TRS-FMED-006`) lists the features the diagram currently shows against every stored `Configuration`: ✓ chosen on, ✗ chosen off, · not mentioned. Collapsing a subtree in the diagram collapses its rows, and the search box keeps the rows that match and their ancestors. Pick two configurations at the top to **compare** them: rows where they differ are highlighted and the panel lists what only the first selects, what only the second selects, and how many features both or neither select. Click a row to inspect that feature.

### Large models

A model of 2,000 features and 300 constraints is analysed in about 0.2 s, a configurator click is answered in about 0.3 s, and the page paints in about a second (release server, `REQ-TRS-FMED-007`). The analysis limit is 5,000 features. A model of more than 60 features opens collapsed to two levels so only what is visible is laid out; **Expand level** opens one more level at a time and **Expand all** the lot. A diagram too wide to read at once (below 30% zoom) is shown from its first root at a readable zoom instead of fitted into an unreadable speck; search, collapse and the matrix are how a model that size is navigated.

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
