---
type: Configuration
id: CONF-EX-RISCV-001
name: "RiscV + GPS-only product, no watchdog"
status: approved
featureModel: Features
features:
  Features::Platform: true
  Features::Platform::CortexM: false
  Features::Platform::CortexM::Fpu: false
  Features::Platform::RiscV: true
  Features::Sensors: true
  Features::Sensors::IMU: false
  Features::Sensors::GPS: true
  Features::Sensors::Lidar: false
  Features::Wdt: false
  Features::Wdt::WindowMode: false
  Features::OrphanRelocated: false
  Features::DataLink: true
  Features::DataLink::Lora: false
  Features::DataLink::Wifi: true
  Features::Legacy::SafeMode: false
buildOverrides:
  PRODUCT_VARIANT: "riscv-wifi"
---

Selects RiscV, GPS only, no watchdog, WiFi datalink. Exercises the
`Platform::RiscV excludes Features::Wdt::WindowMode` constraint (both
inactive here, so no violation) and the `DataLink::Wifi excludes
Sensors::Lidar` constraint.
