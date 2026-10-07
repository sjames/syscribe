---
id: REQ-TRS-VIS-010
type: Requirement
name: A static SVG writer draws diagrams per section 8.16.5 from pins or the embedded layout, and export-html falls back to a companion SVG, a PlantUML render or a placeholder
status: verified
reqDomain: software
verificationMethod: test
---

`vis::svg` **shall** draw any IR with shapes. A fully pinned IR is drawn from its pins,
computing absolute rectangles from the parent-relative pins (a node without a measured size
gets its Rust-computed size, `REQ-TRS-VIS-017`; a container without one bounds its children);
any other IR is laid out first by the embedded ELK (`REQ-TRS-VIS-016`) and drawn from its
rectangles, routed edge polylines and label positions. The output **shall** conform to spec
§8.16.5: the `sysml:` namespace, a `<g id class sysml:ref>` per shape, a `<path id class
sysml:ref sysml:source sysml:target>` per edge with arrowhead markers, and the shared visual
language of `REQ-TRS-VIS-012`. It **shall** refuse only an IR with no shapes (`SvgError::Empty`);
the CLI and MCP then report `'<qname>' has no shapes to draw`. When `[links]` yields a URL for a
shape's element, that shape's `<g>` **shall** be wrapped exactly as `REQ-TRS-LINK-002`
specifies, and left unwrapped otherwise. Output **shall** be deterministic (declaration order,
and ELK's deterministic layout) and all text XML-escaped.

`export-html` **shall** embed each `Diagram` by the first applicable rule: (1) the IR has
shapes — draw it with `vis::svg`; (2) a companion SVG exists — embed the file; (3) a
PlantUML-rendered `.svg` exists beside the companion `.puml` — embed it; (4) otherwise a
placeholder naming the diagram, its kind and subject.

**Source:** `REQ-TRS-VIS-010` (product model).

**Acceptance criteria:** (a) the SVG of the pinned derived BDD and IBD fixtures matches its
golden snapshot and carries `sysml:ref` on every node and `sysml:source`/`sysml:target` on
every edge; (b) the unpinned derived IBD is laid out and drawn, matching its own golden
snapshot, a partially pinned one draws too, and only an empty IR is `SvgError::Empty`; (c) a
links closure returning a URL yields the `<a xlink:href … href … target="_blank"
rel="noopener">` wrapper and no wrapper otherwise; (d) `export-html` inlines
`<svg … sysml:ref=…>` for a fully pinned manifest diagram and for an unpinned one, and a
placeholder for an empty, companion-less one.
