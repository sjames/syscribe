---
type: PartDef
name: Firmware
domain: software
isDeploymentPackage: true
features:
  - {name: supervisor, typedBy: System::Software::Supervisor}
  - {name: mppt, typedBy: System::Software::MpptController}
  - {name: gridController, typedBy: System::Software::GridController}
  - {name: protection, typedBy: System::Software::ProtectionManager}
  - {name: profile, typedBy: System::Software::GridProfileManager}
  - {name: arcAnalyser, typedBy: System::Software::ArcFaultAnalyser}
  - {name: insulationMonitor, typedBy: System::Software::InsulationMonitor}
  - {name: stringMonitor, typedBy: System::Software::StringMonitor}
  - {name: comms, typedBy: System::Software::CommsStack}
  - {name: update, typedBy: System::Software::SecureUpdate}
  - {name: selfTest, typedBy: System::Software::SelfTest}
  - {name: logger, typedBy: System::Software::EventLogger}
connections:
  - {from: profile.profile, to: supervisor.profile}
  - {from: profile.profile, to: protection.profile}
  - {from: protection.trip, to: supervisor.trip}
  - {from: arcAnalyser.trip, to: protection.arcTrip}
  - {from: insulationMonitor.result, to: supervisor.insulation}
  - {from: selfTest.fail, to: supervisor.selfTestFail}
  - {from: comms.command, to: supervisor.cmdIn}
  - {from: comms.profileWrite, to: profile.write}
  - {from: comms.image, to: update.image}
  - {from: supervisor.mpptCtl, to: mppt.ctl}
  - {from: supervisor.gridCtl, to: gridController.ctl}
  - {from: mppt.powerRef, to: gridController.powerRef}
  - {from: mppt.stringData, to: stringMonitor.stringData}
  - {from: supervisor.event, to: logger.event}
  - {from: stringMonitor.warning, to: logger.event}
---

The single firmware image for both regions. Fast control tasks (current loop, PLL, MPPT) run from timer interrupts; the supervisor and protection run in a deterministic scheduler on the same core. Region behaviour comes from the grid profile loaded at commissioning.
