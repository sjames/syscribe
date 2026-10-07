---
id: REQ-TRS-VIS-010
type: Requirement
name: A static SVG writer draws fully pinned diagrams per section 8.16.5, and export-html falls back to a companion SVG, a PlantUML render or a placeholder
status: verified
reqDomain: software
verificationMethod: test
---

`vis::svg` **shall** draw an IR whose every node carries a pin, computing absolute rectangles
from the parent-relative pins (a node without a measured size gets one by kind; a container
without one bounds its children), and emit SVG conforming to spec §8.16.5: the `sysml:`
namespace, a `<g id class sysml:ref>` per shape, a `<path id class sysml:ref sysml:source
sysml:target>` per edge with arrowhead markers, and the shared visual language of
`REQ-TRS-VIS-012`. It **shall** refuse (return `None`) when any node lacks a pin and never
compute positions; the CLI and MCP then report `'<qname>' is not fully pinned — open it in the
browser and use Pin all, or export plantuml/mermaid`. When `[links]` yields a URL for a shape's
element, that shape's `<g>` **shall** be wrapped exactly as `REQ-TRS-LINK-002` specifies, and
left unwrapped otherwise. Output **shall** be deterministic (declaration order) and all text
XML-escaped.

`export-html` **shall** embed each `Diagram` by the first applicable rule: (1) the IR is fully
pinned — draw it with `vis::svg`; (2) a companion SVG exists — embed the file; (3) a
PlantUML-rendered `.svg` exists beside the companion `.puml` — embed it; (4) otherwise a
placeholder naming the diagram, its kind and subject.

**Source:** `REQ-TRS-VIS-010` (product model).

**Acceptance criteria:** (a) the SVG of the pinned derived BDD and IBD fixtures matches its
golden snapshot and carries `sysml:ref` on every node and `sysml:source`/`sysml:target` on
every edge; (b) `render_svg` is `None` when one node is unpinned; (c) a links closure returning
a URL yields the `<a xlink:href … href … target="_blank" rel="noopener">` wrapper and no wrapper
otherwise; (d) `export-html` inlines `<svg … sysml:ref=…>` for a fully pinned manifest diagram
and a placeholder for an unpinned, companion-less one.
