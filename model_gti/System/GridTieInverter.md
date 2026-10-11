---
type: PartDef
name: Grid-Tie Inverter
domain: system
satisfies:
  - REQ-GTI-SYS-001
  - REQ-GTI-SYS-002
  - REQ-GTI-SYS-003
  - REQ-GTI-SYS-004
  - REQ-GTI-SYS-014
features:
  - {name: powerStage, typedBy: System::Hardware::PowerStage}
  - {name: dcInput, typedBy: System::Hardware::DcInput}
  - {name: acOutput, typedBy: System::Hardware::AcOutput}
  - {name: sensing, typedBy: System::Hardware::Sensing}
  - {name: controlBoard, typedBy: System::Hardware::ControlBoard}
  - {name: enclosure, typedBy: System::Hardware::Enclosure}
  - {name: firmware, typedBy: System::Software::Firmware}
  - {name: arcFaultFrontEnd, typedBy: System::Hardware::ArcFaultFrontEnd}
  - {name: rapidShutdownDriver, typedBy: System::Hardware::RapidShutdownDriver}
---

Top-level system: PV strings in, three-phase AC out. A DC input stage with per-tracker boost converters feeds a shared DC link; a three-phase bridge with an LCL filter and two series relays connects to the grid. One STM32 controls everything and a hardware overcurrent path protects the stages independently of software.
