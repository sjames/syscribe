---
type: Diagram
name: BadShapeLink
diagramKind: Mermaid
shapes:
  s-alpha:
    ref: NoSuchPackage::Alpha
    link: NoSuchPackage::Alpha
    kind: PartDef
---

This diagram has a shape with a `link:` that does not resolve — should trigger W411.

```mermaid
graph TD
  s-alpha["Alpha"]
  %% ref: NoSuchPackage::Alpha
```
