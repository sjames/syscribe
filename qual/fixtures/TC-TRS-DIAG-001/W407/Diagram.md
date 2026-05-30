---
type: Diagram
name: SVGIdNotInFrontmatter
diagramKind: BDD
svgMode: inline
---

This diagram has an inline SVG with an id that has no matching entry in frontmatter shapes — should trigger W407.

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
  <rect id="orphan-shape" x="10" y="10" width="80" height="60" fill="#dbeafe"/>
</svg>
```
