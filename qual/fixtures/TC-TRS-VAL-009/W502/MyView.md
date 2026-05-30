---
type: View
name: MyView
expose:
  - NonExistent
---

A View element with an `expose:` entry that does not resolve to any known element — should produce W502.
