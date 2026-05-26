---
type: EnumerationDef
name: FlightMode
supertype: Base::DataValue
values:
  - name: hover
  - name: cruise
  - name: loiter
  - name: returnToHome
  - name: emergency
---

Enumeration of UAV flight modes. The flight controller transitions between these modes in response to operator commands and autonomous logic.
