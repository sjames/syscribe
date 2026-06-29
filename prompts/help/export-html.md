# export-html — generate a standalone offline HTML site for the model

Render the whole model as a self-contained, multi-file static HTML site that
works directly from `file://` — no web server, no network access. Every asset
(stylesheet, Mermaid runtime, search) is bundled and referenced by relative
path, so the output directory can be zipped, copied, or opened straight from
disk.

## Usage

    syscribe -m <model> export-html [--out <dir>] [--css <file>]

  --out <dir>     Output directory (default: `html`). Created if missing.
  --css <file>    Use this CSS file verbatim as the site stylesheet instead of
                  the bundled default theme.

## Output layout

    <out>/
      index.html                  model overview + offline search box
      style.css                   the default theme (or your --css file)
      mermaid.min.js              bundled Mermaid runtime (offline)
      search.js                   client-side search over an inline index
      search-index.json           {qname,id,type,name,url} per element
      elements/<sanitized>.html   one page per element
      reports/validation.html     validation findings
      reports/coverage.html       requirement-verification coverage
      reports/traceability.html   requirements × derivedFrom × verifying tests

Element page filenames are the qualified name with `::` and `/` replaced by
`__` (e.g. `Requirements::REQ-FX-001` → `elements/Requirements__REQ-FX-001.html`).

Each element page shows its identity (name, type, qname, stable id, status), a
frontmatter table with cross-references rendered as links, the rendered Markdown
documentation, and an inlined diagram (SVG for block diagrams, client-side
Mermaid for `diagramKind: Mermaid`).

## Offline guarantee

No generated page loads anything over the network. Search works without
`fetch()`: the index is also embedded inline in `index.html` as
`window.SEARCH_INDEX`.

## Theming a custom stylesheet

A `--css` author can target the class names the generator emits:

  .sitenav          the namespace navigation tree (`a.active` = current page)
  .content          the main column
  .badge            the element-type chip (also `.badge-<type-lowercased>`)
  .meta             the identity/status line under the title
  .fm-table         the frontmatter key/value table
  .doc              the rendered Markdown documentation block
  .diagram          the inlined SVG / Mermaid container
  .refs             a resolved cross-reference list
  .search-box       the search input on the index page
  .search-results   the search result list
  pre.mermaid       a Mermaid source block (rendered client-side)
