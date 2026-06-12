---
type: Allocation
name: Alloc
features:
  - name: dup
    type: Allocation
    allocatedFrom: Func
    allocatedTo: Logical
---
...and the same edge again (form 2) -> redundant, W503.
