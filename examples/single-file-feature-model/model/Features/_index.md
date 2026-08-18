---
type: FeatureModel
name: Features
featureTree:
  # ── Platform: mandatory XOR group, 3 levels deep ──────────────────────────
  - name: Platform
    id: FEAT-PLATFORM-001
    mandatory: true
    groupKind: alternative
    doc: "Every product picks exactly one compute platform."
  - name: Platform.CortexM
    id: FEAT-CORTEXM-001
    groupKind: optional
    buildExports:
      - var: ENABLE_CORTEXM
        whenSelected: 1
        whenDeselected: 0
  - name: Platform.CortexM.Fpu
    id: FEAT-FPU-001
    groupKind: optional
    doc: "Hardware floating point unit — only meaningful under CortexM."
  - name: Platform.RiscV
    id: FEAT-RISCV-001
    groupKind: optional

  # ── Sensors: mandatory OR group (>=1 child) ───────────────────────────────
  - name: Sensors
    id: FEAT-SENSORS-001
    mandatory: true
    groupKind: or
  - name: Sensors.IMU
    id: FEAT-IMU-001
    groupKind: optional
  - name: Sensors.GPS
    id: FEAT-GPS-001
    groupKind: optional
  - name: Sensors.Lidar
    id: FEAT-LIDAR-001
    groupKind: optional

  # ── Wdt: optional, typed parameters, a child, and a relocated feature ────
  - name: Wdt
    id: FEAT-WDT-001
    groupKind: optional
    parameters:
      - name: timeoutMs
        type: ScalarValues::Integer
        range: "10..=5000"
        default: 1000
        isRequired: true
        bindingTime: load
        buildVar: WDT_TIMEOUT_MS
      - name: mode
        type: ScalarValues::String
        enumValues: ["window", "simple"]
        value: "simple"
        isFixed: true
  - name: Wdt.WindowMode
    id: FEAT-WDT-WINDOW-001
    groupKind: optional
  - name: OrphanRelocated
    id: FEAT-RELOCATED-001
    groupKind: optional
    parentFeature: Features::Wdt
    doc: "Demonstrates the parentFeature: override — logically a Wdt child despite a top-level dotted name."

  # ── DataLink: mandatory XOR group ─────────────────────────────────────────
  - name: DataLink
    id: FEAT-DATALINK-001
    mandatory: true
    groupKind: alternative
    contributesTo: SystemFeatures::Comms
  - name: DataLink.Lora
    id: FEAT-LORA-001
    groupKind: optional
  - name: DataLink.Wifi
    id: FEAT-WIFI-001
    groupKind: optional

crossTreeConstraints:
  - feature: Sensors.Lidar
    requires: [Platform.CortexM]                   # dotted, relative to this sheet
  - feature: DataLink.Wifi
    excludes: [Sensors.Lidar]                       # dotted, cross-group
  - feature: Wdt
    requires: [FEAT-LEGACY-SAFEMODE]                 # stable id, resolves to a per-file FeatureDef
  - feature: Platform.RiscV
    excludes: [Features::Wdt::WindowMode]            # already-absolute qname form

parameterConstraints:
  - id: PC-WDT-TIMEOUT
    expression: "Features::Wdt.timeoutMs <= 5000"
    appliesWhen: Features::Wdt
    severity: error
---

The whole feature model for this example, authored as one sheet.
`Features::Legacy::SafeMode` (below) stays a per-file `FeatureDef` to
demonstrate the two authoring forms coexisting in one model.
