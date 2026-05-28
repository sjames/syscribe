---
type: ConnectionDef
name: CAN Bus Connection
ends:
  - name: transmitter
    typedBy: System::Interfaces::CANBusPort
  - name: receiver
    typedBy: System::Interfaces::CANBusPort
---

Binary CAN bus connection definition. Connects a CAN transmitter port to a CAN
receiver port over the ISO 11898-2 physical layer at 500 kbit/s.

Used to wire software component CAN output ports to the hardware transceiver
CAN input port within the Engine ECU deployment context.
