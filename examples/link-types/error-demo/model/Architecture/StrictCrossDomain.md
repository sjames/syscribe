---
type: PartDef
name: StrictCrossDomain
domain: software
satisfies: [REQ-ERR-LT-001]     # E313: the built-in satisfies: is never relaxed
links:
  partiallySatisfies: REQ-ERR-LT-001   # no E313 here: relaxed for this named variant only
---

Shows that relaxation is scoped to the declared variant: the plain `satisfies:`
on the same element still raises `E313`.
