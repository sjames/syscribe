# Web Browser

`BROWSER · OVERVIEW`

The web server is a single Rust binary (`syscribe-server`) that parses a model directory, builds an in-memory element graph, and serves a browser-based UI with live file-system watching.

## Starting the server

```bash
cargo run --package syscribe-server -- model/
# INFO  Loaded 111 elements
# INFO  Listening on http://0.0.0.0:3000
```

Pass any path as the first argument. The server watches the directory for changes and pushes diffs to connected clients over WebSocket.

## Stack

| Layer | Technology |
|---|---|
| HTTP server | Axum (Rust) |
| HTML templates | Askama (server-side rendering) |
| Dynamic updates | HTMX (partial-page swaps — no JS framework) |
| Diagram rendering | SVG (server-built) + Mermaid.js (client CDN) |
| File watching | `notify` crate + WebSocket push |

## UI routes

| Path | Description |
|---|---|
| `GET /` | Root — renders the model tree browser |
| `GET /ui/tree?parent=<qname>` | HTMX — returns tree items for a namespace |
| `GET /ui/detail/<qname>` | HTMX — element detail panel |
| `GET /ui/diagram/<qname>` | HTMX — diagram panel (SVG or Mermaid) |

## API routes

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/elements` | List all elements; optional `?type=PartDef` filter |
| `GET` | `/api/elements/<qname>` | Single element JSON |
| `PUT` | `/api/elements/<qname>` | Write element YAML frontmatter (editing) |
| `GET` | `/api/children?qname=<qname>` | Containment tree |
| `GET` | `/api/connections?qname=<qname>` | Connection graph |
| `PATCH` | `/api/diagrams/layout/<qname>` | Persist drag-adjusted layout coordinates |
| `GET` | `/api/validation` | Validation findings JSON |
| `WS` | `/ws` | Live model-change events |

## Diagram rendering

The `diagram` handler dispatches on `diagramKind`:

- **Mermaid** — extracts the ` ```mermaid ` block from the doc body and wraps it in a `<pre class="mermaid">` that Mermaid.js renders client-side on tab activation.
- **All others** — calls `render_diagram()` in `syscribe-model::renderer`, which builds SVG from the element's `shapes`, `edges`, and `layout` frontmatter. Returns a `<div class="diagram-svg-wrapper">` or a `<p class="diagram-empty">` if no layout is defined.

## Layout persistence

Drag a shape in the browser to reposition it. The client sends a `PATCH /api/diagrams/layout/<qname>` with the new coordinates. The server writes the updated `layout:` block back to the `.md` file on disk. The file watcher then reloads the element and notifies all connected clients.
