---
type: InterfaceDef
name: PowerInterface
supertype: Connections::Interface
ends:
  - name: source
    typedBy: Interfaces::PowerPortDef
    multiplicity: "1"
  - name: receiver
    typedBy: Interfaces::PowerPortReceiverDef
    multiplicity: "1"
constraints:
  - name: currentConservation
    expression: "source.current = receiver.current"
    isAsserted: true
---

Interface between a power source and a power consumer. The current conservation constraint enforces Kirchhoff's current law at this junction.
