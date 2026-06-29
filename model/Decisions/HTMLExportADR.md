---
type: ADR
id: ADR-SYS-HTML-001
name: "Full-model HTML export as a self-contained CLI module producing an offline static site"
status: accepted
tags:
  - html-export
  - tooling
---

## Context

The model can be browsed only through the live Axum web server (`syscribe-server`, which needs a
running process and uses HTMX for dynamic loading) or by reading the raw `.md` files. There is no
way to produce a **standalone, shareable, offline** HTML rendering of the whole model — for
publishing to a static host, attaching to a review package, or reading without the toolchain.

Several approaches were considered for generating static HTML:

1. **Reuse `syscribe-server`'s Askama templates** (depend on the server crate, or extract a
   shared `syscribe-html` crate).
2. **A self-contained module in the `syscribe` CLI** with its own static-oriented templates,
   reusing only the already-shared pure functions from `syscribe-model`.
3. **An external static-site generator** (e.g. emit Markdown and run MkDocs).

## Decision

A new **`syscribe export-html`** subcommand, implemented as a **self-contained module in the
`syscribe` CLI crate** (Option 2), writes a multi-file **offline** static website for the whole
model.

- It reuses the already-pure, already-shared `syscribe-model` functions: `renderer::render_diagram`
  (inline SVG), `validator::validate_with_config` (findings + computed reverse indices), and the
  `Resolver` (cross-reference link targets). Markdown is rendered with `pulldown-cmark`.
- Templates are export-specific (Askama), oriented to a **static linked site** (real anchors,
  persistent sidebar nav) rather than the server's HTMX/Axum-shaped fragments.
- Output is **fully offline**: SysML diagrams are inlined as SVG, Mermaid is rendered client-side
  from a **bundled** `mermaid.min.js`, and the default stylesheet plus a small search script are
  bundled into the binary. No CDN or network references appear in the output.
- Styling is **user-overridable**: a `--css <file>` option replaces the bundled default
  stylesheet; every page links a single `style.css`.

## Rationale

**Against reusing the server templates (Option 1):** the server's templates and handlers are
shaped around HTMX lazy-loading and the Axum request/`State` model — the wrong structure for a
static site, where pages must contain real cross-links and a baked-in nav. Depending on
`syscribe-server` would also pull Axum/tokio/rust-embed into the CLI for machinery it would not
use; extracting a shared template crate is a larger refactor than the export warrants. The truly
reusable pieces — `render_diagram` (already in `syscribe-model`) and a small `markdown_to_html`
(pulldown-cmark) — are cheap to use directly.

**Against an external generator (Option 3):** it would add a non-Rust toolchain dependency and
break the single-binary, offline-by-default property the rest of the tool has.

**For the self-contained CLI module (Option 2):** keeps the export a pure, synchronous,
single-binary capability with no server dependency; gives full control over a static-appropriate
page structure; and reuses the existing model/render/validate logic so the export cannot drift
from the canonical model semantics.

## Consequences

- New dependencies enter the `syscribe` crate: `pulldown-cmark` and `askama`.
- The coverage computation currently inline in the MCP `coverage` tool is extracted into a shared
  `coverage_summary` function reused by both the MCP tool and the HTML coverage report (DRY).
- Output is a flat-ish directory (`index.html`, `style.css`, bundled JS, `elements/`, `reports/`)
  openable directly in a browser with no server and no network.
- Projecting a variant (`--config`) is a natural future extension (out of the initial scope).
