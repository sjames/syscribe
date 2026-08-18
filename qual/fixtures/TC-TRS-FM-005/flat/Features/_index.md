---
type: FeatureModel
name: Features
featureTree:
  - name: Platform
    id: FEAT-FM5-PLATFORM
    mandatory: true
    groupKind: alternative
    doc: "Platform mandatory XOR group, authored as one flat sheet (REQ-TRS-FM-005)."
  - name: Platform.CortexM
    id: FEAT-FM5-CORTEXM
    groupKind: optional
  - name: Platform.RiscV
    id: FEAT-FM5-RISCV
    groupKind: optional
  - name: Wdt
    id: FEAT-FM5-WDT
    groupKind: optional
    parameters:
      - { name: timeoutMs, type: ScalarValues::Integer, range: "10..=5000", default: 1000 }
crossTreeConstraints:
  - feature: Wdt
    requires: [Platform.CortexM]
parameterConstraints:
  - id: PC-FM5-001
    expression: "Features::Wdt.timeoutMs <= 5000"
    appliesWhen: Features::Wdt
    severity: error
---

Whole feature model for this fixture, authored as one flat `featureTree:` sheet
with cross-tree constraints and a sheet-level `parameterConstraints:` entry.
