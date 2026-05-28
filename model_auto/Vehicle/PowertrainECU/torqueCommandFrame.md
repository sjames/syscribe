---
type: Item
name: Torque Command Frame
typedBy: System::Interfaces::CANFrame
---

Item flow representing the torque command CAN frame transmitted from the
ThrottleControl software component to the powertrain CAN network via the
CAN transceiver. Arbitration ID 0x0C8, 8-byte payload, 10 ms cycle time.

Message authentication per `SC-ENG-001` (CMAC-AES-128, 24-bit truncated MAC).
