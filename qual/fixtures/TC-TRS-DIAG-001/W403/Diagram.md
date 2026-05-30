---
type: Diagram
name: BadEdgeRef
diagramKind: BDD
svgMode: inline
shapes:
  s-alpha:
    ref: NoSuchPackage::Alpha
    kind: PartDef
edges:
  e-bad:
    ref: NoSuchPackage::Alpha
    source: s-alpha
    target: undefined-shape
    kind: composition
---

This diagram has an edge whose target references a shape id that is not defined in shapes — should trigger W403.

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
  <rect id="s-alpha" x="10" y="10" width="80" height="60" fill="#dbeafe"/>
  <rect id="e-bad" x="110" y="10" width="80" height="60" fill="#dbeafe"/>
</svg>
```
