---
type: Diagram
name: MermaidNoRefAnnotations
diagramKind: Mermaid
---

This diagram has a Mermaid block but no `%% ref:` annotations — should trigger W409.

```mermaid
graph TD
  A["ComponentA"]
  B["ComponentB"]
  A --> B
```
