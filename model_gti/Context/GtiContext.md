---
type: PartDef
name: Inverter Context
features:
  - {name: inverter, typedBy: System::GridTieInverter}
  - {name: pvArray, typedBy: Context::PvArray, role: external}
  - {name: grid, typedBy: Context::UtilityGrid, role: external}
  - {name: owner, typedBy: Context::OwnerHeadEnd, role: external}
  - {name: installer, typedBy: Context::InstallerTool, role: external}
  - {name: rsdInitiator, typedBy: Context::RapidShutdownInitiator, role: external}
connections:
  - {from: pvArray.dcOut, to: inverter.pvIn}
  - {from: inverter.gridOut, to: grid.acIn}
  - {from: owner.link, to: inverter.comms}
  - {from: installer.link, to: inverter.comms}
  - {from: rsdInitiator.cmd, to: inverter.rsdIn}
---

The inverter and the actors outside it. Every boundary port of the inverter is connected to an actor here, so a port added to the inverter without saying who is on the other side is warning `W996`. The diagrams `GtiContextBdd` and `GtiContextIbd` are derived from this element.
