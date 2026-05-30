---
type: Diagram
name: BadShapeRef
diagramKind: BDD
svgMode: inline
shapes:
  s-ghost:
    ref: NoSuchPackage::NoSuchElement
    kind: PartDef
---

This diagram has a shape whose ref does not resolve and has no resolvable ancestor — should trigger W402.

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
  <rect id="s-ghost" x="10" y="10" width="80" height="60" fill="#dbeafe"/>
</svg>
```
