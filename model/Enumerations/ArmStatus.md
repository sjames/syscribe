---
type: EnumerationDef
name: ArmStatus
supertype: Base::DataValue
values:
  - name: disarmed
  - name: armed
  - name: fault
---

UAV arming state. Motors may only spin when status is armed.
