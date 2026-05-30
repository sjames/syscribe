---
type: PartDef
name: AllocHost
features:
  - name: myAlloc
    type: Allocation
    allocatedFrom: AllocHost
    allocatedTo: NonExistentElement
---

PartDef with an inline Allocation feature whose `allocatedTo` references a non-existent element — should produce E501.
