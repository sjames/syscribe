---
type: Zone
id: ZN-ENG-001
name: Powertrain Control Zone
status: approved
targetSL: 3
achievedSL: 3
members:
  - System::Software::SafetyMonitor
  - System::Software::ThrottleControl
  - System::Hardware::WatchdogTimer
rationale: Hosts the ASIL D throttle control chain; loss of integrity could cause unintended acceleration.
---

The zone holding the safety-critical torque path of the Engine ECU.
