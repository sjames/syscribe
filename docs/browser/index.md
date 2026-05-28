# Web Browser

`BROWSER · OVERVIEW`

The web server is a single Rust binary (`syscribe-server`) that parses a model directory, builds an in-memory element graph, and serves a browser-based UI with live file-system watching.

## Starting the server

```bash
cargo run --package syscribe-server -- model/
# INFO  Loaded 111 elements
#
#   Model browser: http://0.0.0.0:3000/
#   Canvas:        http://0.0.0.0:3000/canvas
```

Pass any path as the first argument. The server watches the directory for changes and pushes diffs to connected clients over WebSocket.

## Stack

| Layer | Technology |
|---|---|
| HTTP server | Axum (Rust) |
| HTML templates | Askama (server-side rendering) |
| Dynamic updates | HTMX (partial-page swaps — no JS framework) |
| Diagram rendering | SVG (server-built) + Mermaid.js (client CDN) |
| Graph canvas | Cytoscape.js + cytoscape-dagre |
| File watching | `notify` crate + WebSocket push |

## UI routes

| Path | Description |
|---|---|
| `GET /` | Root — renders the model tree browser |
| `GET /canvas` | Interactive graph canvas (Cytoscape.js) |
| `GET /ui/tree?parent=<qname>` | HTMX — returns tree items for a namespace |
| `GET /ui/detail/<qname>` | HTMX — element detail panel (rendered Markdown + validation findings) |
| `GET /ui/diagram/<qname>` | HTMX — diagram panel (SVG or Mermaid) |

## API routes

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/elements` | List all elements; optional `?type=PartDef` filter |
| `GET` | `/api/elements/<qname>` | Single element JSON |
| `PUT` | `/api/elements/<qname>` | Write element YAML frontmatter (editing) |
| `GET` | `/api/children?qname=<qname>` | Containment tree |
| `GET` | `/api/connections?qname=<qname>` | Connection graph |
| `GET` | `/api/graph` | Full model graph in Cytoscape.js node/edge format |
| `PATCH` | `/api/diagrams/layout/<qname>` | Persist drag-adjusted layout coordinates |
| `GET` | `/api/validation` | Validation findings JSON (includes `qname` per finding) |
| `WS` | `/ws` | Live model-change events |

## Model graph canvas (`/canvas`)

The canvas is a read-only, interactive graph of the full model rendered with Cytoscape.js. It loads the complete element graph from `/api/graph` and lays it out with the dagre algorithm (left-to-right direction).

### Layout

Elements are arranged in three logical columns:

- **Left** — Requirements, grouped by derivation depth (parent reqs left, derived children to the right of their parents)
- **Middle** — All other model elements (PartDefs, Actions, Allocations, ADRs, etc.)
- **Right** — TestCases

Package namespaces are rendered as compound (container) nodes with a dashed blue border.

### Navigation

- **Pan** — click-drag on the canvas background
- **Zoom** — mouse scroll wheel, pinch-to-zoom on touch screens, or the `+` / `−` toolbar buttons
- **Fit** — press `F` or click the **Fit** button to fit all visible nodes
- **Focus mode** — click any non-package node to zoom into its immediate neighbourhood. Requirements show their full derivation subtree plus satisfying elements and TestCases. Press `← Full graph` or click the background to return.

### Validation highlighting

Nodes with validation findings are given a coloured border:

| Border colour | Meaning |
|---|---|
| Red (4 px) | One or more **errors** |
| Amber (3 px) | **Warnings** only |

Hovering over a highlighted node shows the finding codes in the tooltip (e.g. `⚠ W305, W002`).

The **Issues** toolbar button hides all clean nodes so you can focus only on elements that have findings.

### Element detail panel

Clicking a node opens a side panel that shows:

1. **Element metadata** — name, type badge, qualified name, and stable ID (for `Requirement`, `TestCase`, `ADR`, and all safety analysis types)
2. **Documentation** — full rendered Markdown, including tables, code blocks, and embedded Mermaid diagrams
3. **Validation findings** — any errors (red) or warnings (amber) for that element, showing the code and full message text

The stable ID (e.g. `REQ-FC-001`) is shown in a monospace chip below the qualified name whenever it is set. It is also included in the hover tooltip on the canvas graph.

### Toolbar controls

| Control | Action |
|---|---|
| Search box | Dims nodes whose name or type does not match the query |
| Type filter chips | Hide/show nodes of a specific element type |
| **Issues** | Toggle — show only nodes with errors or warnings |
| **Hide Packages** | Toggle — hide package compound containers |
| `+` / `−` | Zoom in / out |
| **Fit** | Fit all visible nodes into the viewport |

## Diagram rendering

The `diagram` handler dispatches on `diagramKind`:

- **Mermaid** — extracts the ` ```mermaid ` block from the doc body and wraps it in a `<pre class="mermaid">` that Mermaid.js renders client-side on tab activation.
- **All others** — calls `render_diagram()` in `syscribe-model::renderer`, which builds SVG from the element's `shapes`, `edges`, and `layout` frontmatter. Returns a `<div class="diagram-svg-wrapper">` or a `<p class="diagram-empty">` if no layout is defined.

## Layout persistence

Drag a shape in the browser to reposition it. The client sends a `PATCH /api/diagrams/layout/<qname>` with the new coordinates. The server writes the updated `layout:` block back to the `.md` file on disk. The file watcher then reloads the element and notifies all connected clients.
