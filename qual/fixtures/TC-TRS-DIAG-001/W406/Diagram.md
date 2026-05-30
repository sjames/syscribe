---
type: Diagram
name: ShapeIdNotInSVG
diagramKind: BDD
svgMode: inline
shapes:
  s-missing:
    ref: NoSuchPackage::Element
    kind: PartDef
---

This diagram has a shape with id `s-missing` in frontmatter but the inline SVG has no element with that id — should trigger W406.

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
  <rect id="s-other" x="10" y="10" width="80" height="60" fill="#dbeafe"/>
</svg>
```
