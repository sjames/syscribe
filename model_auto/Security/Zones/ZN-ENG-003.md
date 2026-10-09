---
type: Zone
id: ZN-ENG-003
name: Powertrain Network Zone
status: approved
targetSL: 2
achievedSL: 2
members:
  - System::Hardware::CANTransceiver
  - System::Software::CANSecurityModule
rationale: The shared powertrain CAN bus; frames can be injected from the OBD-II port.
---

The zone holding the CAN transceiver and the SecOC message authentication.
