---
type: FMEASheet
id: FMEA-ENG-001
title: FMEA — Throttle, Fuel Control and Safety Monitor Subsystem
status: approved
entries:
  - id: FM-ENG-001
    ref: System::Actuators::ThrottleActuator
    failureMode: Throttle plate stuck open at >20 % position
    effect: Unintended engine acceleration — loss of vehicle control
    cause: Mechanical jam due to foreign object or return spring fracture
    fmeaSeverity: 10
    occurrence: 2
    detection: 3
    recommendedAction: Redundant position feedback and spring-loaded fail-safe design

  - id: FM-ENG-002
    ref: System::Actuators::ThrottleActuator
    failureMode: Throttle plate stuck closed (< 5 %)
    effect: Engine produces insufficient torque — vehicle cannot accelerate
    cause: H-bridge driver open circuit or motor coil failure
    fmeaSeverity: 5
    occurrence: 3
    detection: 2
    recommendedAction: Limp-home mode activated; driver warning issued

  - id: FM-ENG-003
    ref: System::Actuators::FuelInjector
    failureMode: Injector stuck open — permanent fuel delivery
    effect: Engine flooding, rich mixture — potential catalyst damage
    cause: Solenoid winding short circuit
    fmeaSeverity: 7
    occurrence: 2
    detection: 2
    recommendedAction: Lambda sensor closed-loop detects within 200 ms; DTC set

  - id: FM-ENG-004
    ref: System::Actuators::FuelInjector
    failureMode: Injector no-pulse — zero fuel delivery on cylinder
    effect: Misfire, lean mixture, reduced power
    cause: Driver circuit open circuit or connector failure
    fmeaSeverity: 5
    occurrence: 3
    detection: 1
    recommendedAction: Misfire monitor detects within 100 ms; cylinder deactivation

  - id: FM-ENG-005
    ref: System::Software::ThrottleControl
    failureMode: PID integrator windup — runaway throttle command
    effect: Throttle commanded to maximum position
    cause: Actuator position feedback loss with integral action continuing
    fmeaSeverity: 9
    occurrence: 2
    detection: 2
    recommendedAction: Anti-windup logic and safety monitor cross-check in SafetyMonitor SWC

  - id: FM-ENG-006
    ref: System::Software::SafetyMonitor
    failureMode: Safety monitor software crash — no fault detection
    effect: Single-point safety faults go undetected; hardware watchdog remains
    cause: Stack overflow or unhandled exception in safety monitor task
    fmeaSeverity: 9
    occurrence: 1
    detection: 2
    recommendedAction: Hardware watchdog (REQ-ENG-SAFE-002) provides independent backup detection

  - id: FM-ENG-007
    ref: System::Hardware::WatchdogTimer
    failureMode: Watchdog fails to reset on timeout
    effect: Software lock-up not recovered — safety monitor offline
    cause: Watchdog output driver open circuit or reset line disconnected
    fmeaSeverity: 9
    occurrence: 1
    detection: 3
    recommendedAction: Watchdog output continuity test at EOL; independent power supply monitoring DTC

  - id: FM-ENG-008
    ref: System::Sensors::CrankshaftPositionSensor
    failureMode: CPS signal intermittent — signal present but missing teeth
    effect: Engine timing desynchronised — rough running and potential misfires
    cause: Target wheel tooth damage or air gap out of tolerance
    fmeaSeverity: 6
    occurrence: 3
    detection: 2
    recommendedAction: Tooth count plausibility check in EngineStallMonitor; DTC set after 3 consecutive errors

  - id: FM-ENG-009
    ref: System::Software::FuelControl
    failureMode: Lambda feedback loop saturated — fuel trim at maximum positive correction
    effect: Rich mixture at steady state — increased emissions, potential catalyst overheat
    cause: Injector flow degradation or lambda sensor bias error
    fmeaSeverity: 6
    occurrence: 2
    detection: 1
    recommendedAction: Fuel trim range monitoring; DTC set when trim exceeds ±25% for > 30 s

  - id: FM-ENG-010
    ref: System::Software::SafetyMonitor
    failureMode: False fault assertion — spurious safe-state transition
    effect: Unnecessary engine shutdown — driver stranded
    cause: EMC-induced ADC noise spike read as TPS divergence
    fmeaSeverity: 4
    occurrence: 2
    detection: 3
    recommendedAction: Debounce filter (3 consecutive samples) before fault assertion; distinction between transient and latched faults
---

## Scope

Failure mode analysis covering the electronic throttle actuator, fuel injectors,
throttle control software, and safety monitor software. All entries referenced
to their primary architectural element.

## Methodology

FMEA per IEC 60812 / SAE J1739. Severity, occurrence, and detection ratings
on a 1–10 scale. RPN = S × O × D. Entries with RPN > 100 require a documented
recommended action.
