---
type: Configuration
id: CONF-EX-CORTEXM-001
name: "CortexM + Lidar + watchdog product"
status: approved
featureModel: Features
features:
  Features::Platform: true
  Features::Platform::CortexM: true
  Features::Platform::CortexM::Fpu: true
  Features::Platform::RiscV: false
  Features::Sensors: true
  Features::Sensors::IMU: true
  Features::Sensors::GPS: false
  Features::Sensors::Lidar: true
  Features::Wdt: true
  Features::Wdt::WindowMode: true
  Features::OrphanRelocated: true
  Features::DataLink: true
  Features::DataLink::Lora: true
  Features::DataLink::Wifi: false
  Features::Legacy::SafeMode: true
parameterBindings:
  Features::Wdt.timeoutMs: 2000
buildOverrides:
  PRODUCT_VARIANT: "cortexm-lidar"
---

Selects CortexM + FPU, IMU + Lidar, the watchdog (2000ms, window mode) with
SafeMode forced on by the cross-tree constraint, the relocated feature
(logically a Wdt child via `parentFeature:`), and LoRa datalink.
