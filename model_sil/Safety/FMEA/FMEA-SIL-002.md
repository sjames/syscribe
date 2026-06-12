---
id: FMEA-SIL-002
type: FMEASheet
name: FMEA — Trackside Field Equipment
status: approved
entries:
  - id: FM-SIL-009
    ref: System::Hardware::TrackCircuitInterface
    failureMode: Shunt failure — train present but TC reports clear
    effect: Section occupancy undetected — signal may be cleared into occupied section
    cause: High ballast resistance, insulated joint failure, or wheel/rail contact film
    fmeaSeverity: 10
    occurrence: 2
    detection: 2
    recommendedAction: Two independent TC frequencies on adjacent sections provide approach locking; AWS/TPWS as independent train protection layer

  - id: FM-SIL-010
    ref: System::Hardware::TrackCircuitInterface
    failureMode: TC fails permanent-occupied — falsely reports train present
    effect: Section locked out of use; platform inaccessible
    cause: Broken rail bond, receiver circuit failure
    fmeaSeverity: 3
    occurrence: 3
    detection: 1
    recommendedAction: Safe-side failure; maintainer alerted by DTC; section cannot be used until repaired

  - id: FM-SIL-011
    ref: System::Hardware::PointsDriveModule
    failureMode: Points motor stall — motor drives but plate does not move
    effect: Points remain in current position; detected by missing detection confirmation; signal not cleared
    cause: Mechanical obstruction or motor winding failure
    fmeaSeverity: 3
    occurrence: 2
    detection: 1
    recommendedAction: Safe-side failure; detection timeout triggers fault indication; second move attempt after 5 s, then lockout

  - id: FM-SIL-012
    ref: System::Hardware::PointsDriveModule
    failureMode: Points detection contact welded — false position confirmation
    effect: Signal cleared with points not at commanded position — derailment risk
    cause: High contact current welding the detection relay contacts
    fmeaSeverity: 10
    occurrence: 1
    detection: 3
    recommendedAction: Series architecture with two independent detection contacts; contact resistance monitoring; drive and detect circuit separation by separate cable

  - id: FM-SIL-013
    ref: System::Hardware::SignalOutputModule
    failureMode: Signal lamp stick — aspect illuminated when not commanded
    effect: Driver sees a proceed aspect when signal should be red — potential overrun
    cause: Output relay contact welded; short circuit on lamp wiring
    fmeaSeverity: 8
    occurrence: 2
    detection: 2
    recommendedAction: Vital relay architecture with multiple relay contacts in series per aspect; lamp current monitoring detects short; periodic relay test cycle

  - id: FM-SIL-014
    ref: System::Hardware::LevelCrossingModule
    failureMode: Barrier position sensor failure — barriers up but reported as down
    effect: Train signal cleared with barriers not lowered — road/rail collision risk
    cause: Encoder failure or limit switch failure
    fmeaSeverity: 9
    occurrence: 2
    detection: 3
    recommendedAction: Two independent barrier detection circuits (primary and secondary) per barrier; both must confirm lowered position; flashing-light activation independent of signal clearance
---

Scope: trackside field equipment including track circuits (TrackCircuitInterface), points machines (PointsDriveModule), signal heads (SignalOutputModule), and level crossing barriers (LevelCrossingModule). FMEA conducted per IEC 60812.

**Severity scale:** 1 = no effect; 10 = catastrophic.

**Occurrence scale:** 1 = extremely unlikely; 10 = very likely.

**Detection scale:** 1 = near-certain detection before effect; 10 = undetectable.

**Key findings.**

**FM-SIL-009 (Track circuit shunt failure)** has the highest severity (10) in this sheet and is the dominant contributor to FT-SIL-002 (FTE-SIL-004). The detection rating of 2 reflects the approach locking and AWS/TPWS mitigation layers, but these are independent safety layers rather than interlocking detections. The recommended action notes that track circuit reliability is a known systemic challenge; the SIL 4 target for the CBI requires the trackside TC equipment to be certified to EN 50126/50129 with a quantified dangerous failure rate.

**FM-SIL-012 (Points detection contact welded)** is the most critical points failure mode (severity 10). The series two-contact architecture reduces occurrence from 2 to 1 but the detection rating of 3 reflects the difficulty of detecting a welded contact during normal operation (the contact appears to function correctly until a subsequent operation fails to achieve confirmation). Contact resistance monitoring and periodic dry-run testing are the primary detection mechanisms.

**FM-SIL-013 (Signal lamp stick)** is a partially self-detecting failure — a welded relay contact in the "closed" (signal illuminated) position will be detected by the vital relay test cycle if the test cycle exercises the relay to open. The detection rating of 2 reflects this. However, a welded contact on a signal currently displaying a proceed aspect may not be detected until the next time the signal is commanded to red, making this a time-limited exposure failure.

**Safe-side failures.** FM-SIL-010 (TC permanent-occupied) and FM-SIL-011 (points motor stall) are safe-side failures — they cause service disruption but not safety hazard. They are included in the FMEA for completeness and availability analysis but do not contribute to safety goal violations.
