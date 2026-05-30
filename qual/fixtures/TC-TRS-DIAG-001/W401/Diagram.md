---
type: Diagram
name: BadSubject
diagramKind: Mermaid
subject: NonExistentElement
---

This diagram has a subject that does not resolve — should trigger W401.

```mermaid
graph TD
  A --> B
  %% ref: NonExistentElement
```
