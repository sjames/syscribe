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
| Diagram rendering | SVG (server-built) + Mermaid.js; an editable sprotty-based diagram client for non-Mermaid diagrams |
| File watching | `notify` crate + WebSocket push |

All JavaScript (HTMX, Mermaid, the bundled diagram editor) is vendored and served from the binary under `/static/` — no CDN is needed at runtime.

## UI routes

| Path | Description |
|---|---|
| `GET /` | Root — renders the model tree browser |
| `GET /ui/tree?parent=<qname>` | HTMX — returns tree items for a namespace (top level when `parent` is omitted) |
| `GET /ui/detail/<qname>` | HTMX — element detail panel (rendered Markdown, custom fields, generated member list, edit form) |
| `GET /ui/diagram/<qname>` | HTMX — diagram panel (SVG or Mermaid) |
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
| `GET` | `/api/diagrams/model/<qname>` | A `Diagram` element as a sprotty graph model (nodes from `shapes:`/`layout:`, edges from `edges:`) |
| `PATCH` | `/api/diagrams/layout/<qname>` | Persist drag-adjusted layout coordinates (guarded write) |
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

The `diagram` handler dispatches on `diagramKind`:

- **Mermaid** — extracts the ` ```mermaid ` block from the doc body and wraps it in a `<pre class="mermaid">` that Mermaid.js renders client-side on tab activation.
- **All others** — calls `render_diagram()` in `syscribe-model::renderer`, which builds SVG from the element's `shapes`, `edges`, and `layout` frontmatter. Returns a `<div class="diagram-svg-wrapper">` or a `<p class="diagram-empty">` if no layout is defined.

## Layout persistence

Drag a shape in the browser to reposition it. The client sends a `PATCH /api/diagrams/layout/<qname>` with the new coordinates. The server writes the updated `layout:` block back to the `.md` file on disk. The file watcher then reloads the element and notifies all connected clients.
