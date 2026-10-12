---
type: FMEASheet
id: FMEA-GTI-002
name: "FMEA — Grid disconnection chain (injectable)"
status: draft
entries:
  - id: FM-GTI-001
    ref: Behavior::Parts::GridMonitorPart
    failureMode: "Software grid monitor fails silent — no GridLost report"
    effect: "Supervisor never commands shutdown; only the hardware trip remains"
    cause: "Firmware task starvation, hang, or corrupted state"
    fmeaSeverity: 9
    occurrence: 3
    detection: 3
    recommendedAction: "Independent hardware trip; window watchdog on the monitor task"
    injection: [{kind: silent, part: monitor}]
  - id: FM-GTI-002
    ref: Behavior::Parts::GridMonitorPart
    failureMode: "Monitor report lost on its way to the supervisor"
    effect: "As FM-GTI-001"
    cause: "Message queue overflow or inter-task fault"
    fmeaSeverity: 9
    occurrence: 2
    detection: 3
    recommendedAction: "Supervisor timeout on monitor heartbeat"
    injection: [{kind: drop, port: out, part: monitor}]
  - id: FM-GTI-003
    ref: System::Software::Supervisor
    failureMode: "Supervisor fails silent — relays and gates are not commanded"
    effect: "Relay A stays closed and the bridge keeps switching unless the hardware trip acts"
    cause: "Firmware hang"
    fmeaSeverity: 9
    occurrence: 3
    detection: 3
    recommendedAction: "Hardware trip opens relay B and disables the gates"
    injection: [{kind: silent, part: supervisor}]
  - id: FM-GTI-004
    ref: System::Software::Supervisor
    failureMode: "Relay commands lost between supervisor and relay drivers"
    effect: "Relays are not opened by software"
    cause: "Driver or bus fault"
    fmeaSeverity: 9
    occurrence: 2
    detection: 4
    recommendedAction: "Auxiliary-contact feedback and hardware trip"
    injection: [{kind: drop, port: relayCmd, part: supervisor}]
  - id: FM-GTI-005
    ref: Behavior::Parts::HwTripPart
    failureMode: "Hardware trip fails silent (comparator or latch failure)"
    effect: "Only the software path can disconnect"
    cause: "Comparator drift, supply fault, latch failure"
    fmeaSeverity: 9
    occurrence: 3
    detection: 6
    recommendedAction: "Periodic self-test of the trip path at start-up"
    injection: [{kind: silent}]
  - id: FM-GTI-006
    ref: Behavior::Parts::RelayPart
    failureMode: "Relay A contacts welded closed"
    effect: "Relay A cannot isolate"
    cause: "Contact welding by inrush or fault current"
    fmeaSeverity: 9
    occurrence: 3
    detection: 4
    recommendedAction: "Series relay B; weld detection by auxiliary contact before every connection"
    injection: [{kind: stuck, variable: closed, value: "1", part: relayA}]
  - id: FM-GTI-007
    ref: Behavior::Parts::RelayPart
    failureMode: "Relay B contacts welded closed"
    effect: "Relay B cannot isolate"
    cause: "Contact welding"
    fmeaSeverity: 9
    occurrence: 3
    detection: 4
    recommendedAction: "Series relay A; weld detection"
    injection: [{kind: stuck, variable: closed, value: "1", part: relayB}]
  - id: FM-GTI-008
    ref: Behavior::Parts::PowerStagePart
    failureMode: "Gate drive stuck on — bridge keeps switching"
    effect: "Bridge energises its output while the relays are closed"
    cause: "Gate-driver or PWM output failure"
    fmeaSeverity: 9
    occurrence: 3
    detection: 5
    recommendedAction: "Relays open on grid loss; hardware gate disable"
    injection: [{kind: stuck, variable: pwm, value: "1"}]
  - id: FM-GTI-009
    ref: Behavior::Parts::SensorPart
    failureMode: "Software-path voltage sensing reads nominal while the grid is lost"
    effect: "The monitor's voltage window does not see the loss; the perturbation-response check (REQ-GTI-SW-008) and the hardware trip still can"
    cause: "Sensor or isolation amplifier stuck"
    fmeaSeverity: 9
    occurrence: 3
    detection: 7
    recommendedAction: "Independent sensing chain for the hardware trip (REQ-GTI-HW-008, ADR-GTI-SAFE-001)"
    injection: [{kind: corrupt, port: out, field: v, value: "1.0", part: sensorSw}]
  - id: FM-GTI-011
    ref: Behavior::Parts::SensorPart
    failureMode: "Hardware-trip voltage sensing reads nominal while the grid is lost"
    effect: "The hardware trip does not see the loss; the software path still can"
    cause: "Comparator input network stuck"
    fmeaSeverity: 9
    occurrence: 3
    detection: 6
    recommendedAction: "Start-up test of the hardware trip with an injected reference"
    injection: [{kind: corrupt, port: out, field: v, value: "1.0", part: sensorHw}]
  - id: FM-GTI-010
    ref: Behavior::Parts::GridMonitorPart
    failureMode: "Monitor trip threshold drifts (prose only — an analysis row without a fault model)"
    effect: "Late or early trip"
    cause: "Reference drift"
    fmeaSeverity: 5
    occurrence: 3
    detection: 4
    recommendedAction: "Calibrate at commissioning"
---

Failure modes of the grid-disconnection chain (SG-GTI-001) with the fault model that makes each one injectable into the executable composite `Behavior::Chain::ProtectionChain`. Occurrence and detection ratings are proposals for review.
