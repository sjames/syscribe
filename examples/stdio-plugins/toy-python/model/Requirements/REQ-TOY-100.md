---
type: Requirement
id: REQ-TOY-100
name: "Coolant flow shall be regulated"
status: approved
reqDomain: software
reqClass: system
---

The system shall regulate coolant flow to keep component temperatures within
their rated operating range. Satisfied by `Legacy::FlowController`, a
plugin-synthesized element — see `../Legacy/widgets.toy`.
