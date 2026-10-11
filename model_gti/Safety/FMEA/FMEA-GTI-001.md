---
type: FMEASheet
id: FMEA-GTI-001
name: "FMEA — Inverter architecture (prose view)"
status: draft
entries:
  - id: FM-GTI-101
    ref: System::Software::ArcFaultAnalyser
    appliesWhen: Features::Protection::Afci
    failureMode: "Arc-fault analyser: arc not recognised (false negative) on a degraded connector"
    effect: "Sustained series arc on a 1000 V string; fire risk"
    cause: "Signature of the arc falls below the detection threshold, or the front end saturates"
    fmeaSeverity: 9
    occurrence: 3
    detection: 6
    recommendedAction: "Test with UL 1699B arc generator; self-test injection of a synthetic signature at start-up"
  - id: FM-GTI-102
    ref: System::Software::ArcFaultAnalyser
    appliesWhen: Features::Protection::Afci
    failureMode: "Arc-fault analyser: nuisance trip (false positive) from inverter switching noise or lightning"
    effect: "Lost energy yield; service call"
    cause: "Threshold too low in a noisy installation"
    fmeaSeverity: 4
    occurrence: 5
    detection: 3
    recommendedAction: "Adaptive threshold; trip counter with automatic retry limited per UL 1699B"
  - id: FM-GTI-103
    ref: System::Hardware::Sensing
    failureMode: "Insulation measurement network fails open — always reads infinite resistance"
    effect: "A leaking array is connected through a transformerless inverter"
    cause: "Open resistor or relay in the measurement network"
    fmeaSeverity: 8
    occurrence: 3
    detection: 5
    recommendedAction: "Plausibility check against a reference resistor at start-up; residual-current monitor as second layer"
  - id: FM-GTI-104
    ref: System::Hardware::DcInput
    failureMode: "String fuse fails to open on reverse current"
    effect: "Reverse current heats a faulted string"
    cause: "Fuse rated for the wrong voltage or current"
    fmeaSeverity: 8
    occurrence: 2
    detection: 4
    recommendedAction: "Fuse rated for 1000 V DC gPV; verified by inspection and test"
  - id: FM-GTI-105
    ref: System::Software::MpptController
    failureMode: "Tracker locks on a local maximum under partial shading"
    effect: "Energy loss up to tens of percent on shaded arrays"
    cause: "Perturb-and-observe cannot cross a valley on the P-V curve"
    fmeaSeverity: 4
    occurrence: 6
    detection: 5
    recommendedAction: "Periodic global scan (REQ-GTI-SYS-023); scenarios in TC-GTI-MPPT-001"
  - id: FM-GTI-106
    ref: System::Hardware::Stm32G474
    failureMode: "Clock failure or flash corruption in the STM32"
    effect: "Control loops stop or misbehave"
    cause: "Crystal failure, bit flip"
    fmeaSeverity: 7
    occurrence: 3
    detection: 3
    recommendedAction: "Clock security system, flash CRC and window watchdog (REQ-GTI-SYS-071); hardware trip stays independent"
  - id: FM-GTI-107
    ref: System::Software::GridProfileManager
    failureMode: "Wrong grid profile active (US limits in India or the reverse)"
    effect: "Trip limits and reconnect delay do not match the utility rules; non-compliant export"
    cause: "Commissioning error or unauthorised change"
    fmeaSeverity: 6
    occurrence: 4
    detection: 4
    recommendedAction: "Authenticated commissioning, locked profile, profile shown on local display and logged (REQ-GTI-SYS-060)"
  - id: FM-GTI-108
    ref: System::Hardware::Enclosure
    failureMode: "Heatsink fouled by dust (India) — derating not enough"
    effect: "Over-temperature in the enclosure"
    cause: "Dust and monsoon humidity"
    fmeaSeverity: 6
    occurrence: 5
    detection: 3
    recommendedAction: "Temperature-based derate and shutdown (REQ-GTI-SYS-036); IP65 passive design"
---

Architecture-level FMEA. Ratings are proposals for review. The failures of the grid-disconnection chain that can be injected into the executable model are in `FMEA-GTI-002`.
