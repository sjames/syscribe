---
type: PartDef
name: WatchdogMonitor
domain: software
links:
  mitigates: [REQ-BRK-002, REQ-BRK-003]
tags:
  - brakes
  - safety
---

Independent software monitor that forces the modulator into its pressure-release
state if the brake controller stops refreshing its heartbeat.

`mitigates` is a free-standing link type (no `extends`), so it is checked only
against its own declared constraints — `sourceTypes = ["PartDef"]`,
`targetTypes = ["Requirement"]` — and never against built-in rules such as
`E313`. That is why this software element may mitigate the hardware
requirement `REQ-BRK-003` without any relaxation.
