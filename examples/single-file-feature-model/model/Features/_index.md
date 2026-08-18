---
type: FeatureModel
name: Features
featureTree:
  # ── Platform: mandatory XOR group, 3 levels deep ──────────────────────────
  # No id: on most entries below — REQ-TRS-FM-006 derives one from the dotted
  # name (e.g. Platform.CortexM.Fpu -> FEAT-PLATFORM-CORTEXM-FPU). Wdt keeps
  # an explicit id: to demonstrate that it always overrides derivation.
  - name: Platform
    mandatory: true
    groupKind: alternative
    doc: "Every product picks exactly one compute platform."
  - name: Platform.CortexM
    groupKind: optional
    buildExports:
      - var: ENABLE_CORTEXM
        whenSelected: 1
        whenDeselected: 0
  - name: Platform.CortexM.Fpu
    groupKind: optional
    doc: "Hardware floating point unit — only meaningful under CortexM."
  - name: Platform.RiscV
    groupKind: optional

  # ── Sensors: mandatory OR group (>=1 child) ───────────────────────────────
  - name: Sensors
    mandatory: true
    groupKind: or
  - name: Sensors.IMU
    groupKind: optional
  - name: Sensors.GPS
    groupKind: optional
  - name: Sensors.Lidar
    groupKind: optional

  # ── Wdt: optional, typed parameters, a child, and a relocated feature ────
  - name: Wdt
    id: FEAT-WDT-PINNED      # explicit id: — kept stable across any future rename
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
    groupKind: optional
  - name: OrphanRelocated
    id: FEAT-RELOCATED      # explicit id: needed — "OrphanRelocated" derives to a
                            # 15-char segment, over the FEAT-* pattern's 12-char cap (E006)
    groupKind: optional
    parentFeature: Features::Wdt
    doc: "Demonstrates the parentFeature: override — logically a Wdt child despite a top-level dotted name."

  # ── DataLink: mandatory XOR group ─────────────────────────────────────────
  - name: DataLink
    mandatory: true
    groupKind: alternative
    contributesTo: SystemFeatures::Comms
  - name: DataLink.Lora
    groupKind: optional
  - name: DataLink.Wifi
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
