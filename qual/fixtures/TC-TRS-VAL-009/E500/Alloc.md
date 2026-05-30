---
type: PartDef
name: AllocHost
features:
  - name: myAlloc
    type: Allocation
    allocatedFrom: NonExistentElement
    allocatedTo: AllocHost
---

PartDef with an inline Allocation feature whose `allocatedFrom` references a non-existent element — should produce E500.
