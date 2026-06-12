---
type: Allocation
name: Alloc
features:
  - name: funcToLogical
    allocatedFrom: Func
    allocatedTo: Logical
  - name: logicalToPhysical
    allocatedFrom: Logical
    allocatedTo: Physical
---
Legacy standalone allocation; features omit a per-entry `type: Allocation`.
