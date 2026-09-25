---
type: Diagram
name: InlineSvg
diagramKind: BDD
svgMode: inline
subject: Arch::Vehicle
shapes:
  s-vehicle: {ref: "Arch::Vehicle", kind: PartDef}
  s-inline-missing: {ref: "Arch::Engine", kind: PartDef}
---

Hand-authored inline SVG that lacks the `s-inline-missing` id: W406 must fire.

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">
  <rect id="s-vehicle" x="10" y="10" width="80" height="60"/>
</svg>
```
