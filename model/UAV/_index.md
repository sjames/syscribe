---
type: Package
name: UAV
imports:
  - target: ISQ::*
  - target: Interfaces::*
  - target: Items::*
  - target: Flows::*
  - target: Enumerations::*
aliases:
  - name: Mass
    for: ISQ::MassValue
  - name: Power
    for: ISQ::PowerValue
---

Package containing all UAV airframe structural definitions, organized into propulsion, avionics, power, and payload sub-packages.
