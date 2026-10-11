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
  - REQ-GTI-SYS-005
  - REQ-GTI-SYS-042
  - REQ-GTI-SYS-061
  - REQ-GTI-SYS-006
  - REQ-GTI-SYS-043
  - REQ-GTI-SYS-062
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
connections:
  - {from: dcInput.dcOut, to: powerStage.dcIn}
  - {from: powerStage.acOut, to: acOutput.acIn}
  - {from: dcInput.strSense, to: sensing.dcSig}
  - {from: acOutput.acSense, to: sensing.acSig}
  - {from: powerStage.tempOut, to: sensing.tempSig}
  - {from: sensing.adcOut, to: controlBoard.adcIn}
  - {from: sensing.hwVoltage, to: controlBoard.hwVin}
  - {from: arcFaultFrontEnd.sig, to: controlBoard.arcIn}
  - {from: acOutput.relayAux, to: controlBoard.auxIn}
  - {from: controlBoard.pwmOut, to: powerStage.gate}
  - {from: controlBoard.relayCmd, to: acOutput.relayDrive}
  - {from: controlBoard.hwTripGate, to: powerStage.hwDisable}
  - {from: controlBoard.hwTripRelay, to: acOutput.relayTrip}
  - {from: controlBoard.rsdCmd, to: rapidShutdownDriver.cmd}
---

Top-level system: PV strings in, three-phase AC out. A DC input stage with per-tracker boost converters feeds a shared DC link; a three-phase bridge with an LCL filter and two series relays connects to the grid. One STM32 controls everything and a hardware overcurrent path protects the stages independently of software.
