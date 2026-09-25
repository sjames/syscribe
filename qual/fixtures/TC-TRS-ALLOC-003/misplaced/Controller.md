---
type: PartDef
name: Controller
features:
  - name: thrustAlloc
    type: Allocation
    allocatedFrom: ComputeThrust
    allocatedTo: Board
---

A features-form allocation on a PartDef: §12.9 recognises the features form only on a
`type: Allocation` element, so this entry contributes no allocation edge (W930).
