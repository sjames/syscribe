---
type: InterfaceDef
name: Powertrain CAN Interface
---

Interface definition for the ISO 11898-2 powertrain CAN network. Specifies
the protocol contract (500 kbit/s, 11-bit arbitration IDs, bus fault tolerance)
that all nodes connected to the powertrain CAN bus must satisfy.

Any port typed by `CANBusPort` that participates in this interface must implement
message authentication per `SC-ENG-001` for safety-critical PDUs.
