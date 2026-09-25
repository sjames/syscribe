---
type: Package
name: main
---

Fixture for TC-TRS-SEC-009: the attack tree's root gate (ATG-ROOT-002) sorts
**after** a sub-gate (ATG-ROOT-001) in file order, so a roll-up that took the
first gate it met as the root would compute the wrong feasibility.
