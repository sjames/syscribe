---
type: FMEASheet
id: FMEA-ENG-002
title: FMEA — Engine Sensors (CPS, TPS, Lambda)
status: approved
entries:
  - id: FM-ENG-011
    ref: System::Sensors::ThrottlePositionSensor
    failureMode: TPS track 1 / track 2 divergence above 5 %
    effect: Safety monitor asserts TPS fault — fuel cut and limp-home mode activated
    cause: Resistive track wear, wiper contact intermittency, or moisture ingress
    fmeaSeverity: 8
    occurrence: 3
    detection: 1
    recommendedAction: Continuous dual-track comparison in SafetyMonitor; DTC P0122/P0123 set on divergence > 5 % for > 20 ms

  - id: FM-ENG-012
    ref: System::Sensors::ThrottlePositionSensor
    failureMode: TPS both tracks stuck at 0 % (idle position)
    effect: Engine cannot accelerate — vehicle stranded or limp-home only
    cause: Common connector failure, open-circuit supply reference, or mechanical sensor jam at closed stop
    fmeaSeverity: 7
    occurrence: 2
    detection: 2
    recommendedAction: Plausibility check against pedal demand sensor; if pedal > 20 % and TPS at 0 %, set DTC and alert driver

  - id: FM-ENG-013
    ref: System::Sensors::LambdaSensor
    failureMode: Lambda sensor heater failure — sensor remains unheated
    effect: Closed-loop fuelling unavailable; engine runs open-loop with calibrated base map
    cause: Heater element wire fracture or heater driver MOSFET open circuit
    fmeaSeverity: 4
    occurrence: 3
    detection: 1
    recommendedAction: Heater current monitoring; DTC P0030 set within 30 s of insufficient heater current; open-loop flag visible in OBD data

  - id: FM-ENG-014
    ref: System::Sensors::LambdaSensor
    failureMode: Lambda sensor output clamped at lambda = 0.7 (rich bias)
    effect: Rich mixture fuelling at all loads — catalyst overheat risk, poor fuel economy, potential misfire
    cause: Sensor reference air path blocked (contamination of internal air path) or signal circuit bias fault
    fmeaSeverity: 6
    occurrence: 2
    detection: 2
    recommendedAction: Fuel trim monitoring; DTC P0172 (system too rich) set when short-term fuel trim correction < −20 % for > 10 s

  - id: FM-ENG-015
    ref: System::Sensors::CrankshaftPositionSensor
    failureMode: CPS signal inverted polarity — engine timing 180° offset
    effect: Incorrect injection and ignition timing relative to TDC — engine fails to start or stalls immediately
    cause: Reversed connector polarity during sensor replacement, or differential signal pair swapped in harness
    fmeaSeverity: 8
    occurrence: 1
    detection: 2
    recommendedAction: Polarity identification marking on connector (asymmetric key); build-time harness continuity test; cam/crank synchronisation check at first engine start

  - id: FM-ENG-016
    ref: System::Sensors::CrankshaftPositionSensor
    failureMode: CPS target wheel missing 2-tooth gap — timing reference lost
    effect: ECU cannot identify TDC reference cylinder — rough running, random ignition firing, stall risk
    cause: Target wheel installation error (wrong part number) or mechanical tooth damage to the reference gap
    fmeaSeverity: 7
    occurrence: 1
    detection: 2
    recommendedAction: Target wheel part number verification at engine assembly EOL; synchronisation status DTC (P0315) flagged if reference pattern not detected within 4 engine revolutions
---

## Scope

Failure mode analysis covering the three primary engine management sensors:
Crankshaft Position Sensor (CPS), Throttle Position Sensor (TPS — dual-track),
and Lambda (oxygen) sensor. These sensors are critical inputs to the throttle
control, fuel control, and safety monitor software components.

## Methodology

FMEA per IEC 60812 / SAE J1739. Severity, occurrence, and detection ratings
on a 1–10 scale. RPN = S × O × D. Entries with RPN > 100 require a documented
recommended action.

## Relationship to Fault Tree Analysis

FM-ENG-015 and FM-ENG-016 address CPS failure modes that are distinct from the
wire harness and common-cause failure modes covered in FT-ENG-002. FM-ENG-011
(TPS dual-track divergence) is the sensor-level failure mode corresponding to
FTE-ENG-002 in the FT-ENG-001 fault tree for unintended acceleration.

Common-cause analysis for TPS dual-track (FM-ENG-011) was performed in the
FTA using the AND gate FTG-ENG-002. The FMEA entry confirms that the safety
monitor's continuous dual-track comparison provides detection (D = 1) sufficient
to keep the RPN within acceptable bounds for ASIL A.
