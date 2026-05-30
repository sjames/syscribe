---
type: Diagram
name: SVGBadHref
diagramKind: BDD
svgMode: inline
---

This diagram has an inline SVG with a relative href that does not match any model element file — should trigger W412.

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
  <a href="./NoSuchElement.md">
    <rect id="s-box" x="10" y="10" width="80" height="60" fill="#dbeafe"/>
  </a>
</svg>
```
