---
type: PartDef
name: Object Controller
domain: hardware
features:
  - name: fieldInterfaceCount
    typedBy: ScalarValues::Integer
  - name: powerSupplyVoltage
    typedBy: ScalarValues::Real
    unit: V
---

The Object Controller (OC) is a field interface unit that drives and monitors one or more trackside objects, which may include signals, points machines, and track circuit sections. The OC communicates with the vital processor over the EN 50159 Category 2 field bus, receiving commands and sending back status information on every polling cycle.

Each OC contains fail-safe relay logic for its assigned objects. A relay is considered energised (permissive) only when the vital processor actively commands and acknowledges that state; any loss of communication or command timeout causes the relays to de-energise, returning all assigned objects to their safe states.

The OC also provides galvanic isolation between the trackside field circuits and the interlocking electronics, protecting the vital processor from transients induced by the railway environment (traction currents, thunderstorms).

Object Controllers are deployed adjacent to the trackside equipment they serve, reducing cable lengths and improving electromagnetic immunity.
