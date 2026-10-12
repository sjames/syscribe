---
type: Package
name: Interfaces
---

The port types and the events they carry, shared by the architecture ports (`System::`) and the executable parts that model them (`Behavior::Parts::`). A port typed by one of these `PortDef`s can send or accept only the items it lists, in the item's direction (reversed at a port with `isConjugated: true`), and a payload must match the item's fields (GH #331).
