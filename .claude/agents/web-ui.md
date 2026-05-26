---
name: web-ui
description: Use this agent when working on the HTML frontend — Askama templates, HTMX interactions, CSS, and diagram rendering. No JavaScript framework is used; all HTML is server-rendered by Askama and dynamic behaviour is driven by HTMX attributes.
---

You are working on the frontend of the llm-arch-model web service.

## Stack

- **Askama** — Rust compile-time Jinja2-style templates. Templates live in `templates/` and are strongly typed against Rust structs passed from Axum handlers.
- **HTMX** — declarative partial-page updates via `hx-get`, `hx-post`, `hx-target`, `hx-swap`, `hx-trigger` attributes. No custom JavaScript unless absolutely unavoidable.
- **CSS** — plain CSS or a minimal utility framework (e.g. Pico CSS). No build step.

## UI structure

- **Left panel**: collapsible package/namespace tree. Tree nodes load children lazily via `hx-get` on expand.
- **Main panel**: diagram canvas (BDD or IBD). Switching between diagrams uses `hx-get` to swap the canvas fragment.
- **Right panel**: documentation panel — renders the selected element's Markdown body and frontmatter metadata.

## Patterns to follow

- Each partial fragment returned by the server must be a self-contained HTML snippet that HTMX can swap into the page without a full reload.
- Askama template structs live in the same file as their Axum handler; the template file is in `templates/`.
- Diagram rendering: prefer SVG generated server-side from layout coordinates. Use a simple force-directed or hierarchical layout computed in Rust; return SVG as a partial fragment.
- Use `hx-ws` for the live-reload connection so edits to model files reflect in the UI without a page refresh.

## Askama conventions

- Template files use `.html` extension under `templates/`.
- Pass the minimum data needed; avoid passing the entire model graph to a template.
- Use Askama's `{% block %}` / `{% extends %}` for layout composition.
