# Engine ECU — ISO 26262 / ISO/SAE 21434

`EXAMPLE · REFERENCE MODEL · model_auto/`

A complete Engine Control Unit reference model for a passenger vehicle powertrain. It covers system architecture, requirements traceability, functional safety analysis to ISO 26262, and cybersecurity analysis to ISO/SAE 21434 — from top-level safety goals down to verified leaf requirements and HIL test cases.

---

## Model scope

The model covers the **Engine ECU** of a passenger car. The ECU controls the electronic throttle actuator and fuel injectors, reads crankshaft position and throttle position sensors, and communicates with the rest of the vehicle over a powertrain CAN bus. The primary safety concern is *unintended engine acceleration* and *engine stall at high speed*, both rated ASIL D under ISO 26262.

Standards applied:

- **ISO 26262:2018** — Functional safety for road vehicles; safety goals rated ASIL A–D
- **ISO/SAE 21434:2021** — Road vehicle cybersecurity engineering; CAL 1–3 security goals
- **AUTOSAR SecOC** — Message authentication on safety-critical CAN PDUs
- **UDS (ISO 14229)** — Diagnostic protocol with security access levels

## Architecture

```
EngineECU (hardware PartDef)
└── EngineControlSoftware (software PartDef, isDeploymentPackage)
    ├── SafetyMonitor        ← ASIL D — supervises all others
    ├── ThrottleControl      ← PID, anti-windup, limp-home
    ├── FuelControl          ← lambda closed-loop, rev limiter
    ├── EngineStallMonitor   ← CPS tooth-count plausibility
    ├── CANSecurityModule    ← AUTOSAR SecOC MAC verify/generate
    ├── DiagnosticSecurityLayer ← UDS session / security access
    ├── SecureBootManager    ← ECDSA firmware signature verify
    └── BootSequence         ← ActionDef: ordered startup

Deployed as: Vehicle::PowertrainECU
```

---

## Hardware

| Element | Type | Description |
|---|---|---|
| `System::EngineECU` | PartDef (hardware) | Top-level ECU hardware shell; supply voltage, operating temperature, CAN transceiver and watchdog features |
| `System::EngineControlSoftware` | PartDef (software, deploymentPackage) | Deployable software image; allocated to `EngineECU` |
| `System::Sensors::ThrottlePositionSensor` | PartDef | Dual-track resistive TPS; divergence threshold triggers ASIL D fault path |
| `System::Sensors::CrankshaftPositionSensor` | PartDef | Hall-effect CPS; tooth-count plausibility monitored by EngineStallMonitor |
| `System::Sensors::LambdaSensor` | PartDef | Wideband lambda (UEGO); input to FuelControl closed-loop |
| `System::Actuators::ThrottleActuator` | PartDef | H-bridge driven throttle plate; spring-loaded fail-safe at 7 % |
| `System::Actuators::FuelInjector` | PartDef | Solenoid fuel injector; drive-circuit monitored by SafetyMonitor |
| `System::Hardware::WatchdogTimer` | PartDef | Independent hardware watchdog; triggers ECU reset on software lock-up |

## Software components

| Element | ASIL | Description |
|---|---|---|
| `System::Software::SafetyMonitor` | D | Supervises all safety-relevant inputs and outputs; asserts fault signal to ThrottleControl and FuelControl; controlled shutdown within 200 ms |
| `System::Software::ThrottleControl` | — | PID position controller with anti-windup; limp-home mode at 7 % |
| `System::Software::FuelControl` | — | Lambda closed-loop, rev limiter (soft retard at 6200 rpm, hard cut at 6500 rpm) |
| `System::Software::EngineStallMonitor` | — | CPS tooth-count plausibility; DTC after 3 consecutive errors |
| `System::Software::CANSecurityModule` | — | AUTOSAR SecOC MAC generation and verification on powertrain CAN PDUs |
| `System::Software::DiagnosticSecurityLayer` | — | UDS session management; security access level 0x11/0x12 with audit logging |
| `System::Software::SecureBootManager` | — | ECDSA P-256 firmware signature verification before flash programming |

---

## Requirements

### Safety (ISO 26262)

| ID | Title | ASIL |
|---|---|---|
| REQ-ENG-SAFE-001 | Safety monitor shall detect all safety faults within 100 ms | D |
| REQ-ENG-SAFE-002 | Hardware watchdog shall reset ECU within 50 ms of software lock-up | D |
| REQ-ENG-SAFE-003 | TPS dual-track divergence shall trigger safe state within 50 ms | C |
| REQ-ENG-SAFE-004 | Rev limiter shall enforce fuel cut independently of TPS | B |
| REQ-ENG-SAFE-005 | Throttle close command shall be verified by TPS position feedback | D |

### Security (ISO/SAE 21434)

| ID | Title |
|---|---|
| REQ-ENG-SEC-001 | Safety-critical CAN messages shall be authenticated using MAC |
| REQ-ENG-SEC-002 | ECU calibration programming sessions shall require cryptographic authentication |
| REQ-ENG-SEC-003 | Firmware updates shall require ECDSA P-256 signature verification |
| REQ-ENG-SEC-004 | Diagnostic memory read access shall be restricted to authorised sessions |

---

## Safety analysis

**HARA** identifies three safety goals:

| ID | Title | ASIL | FTTI |
|---|---|---|---|
| SG-ENG-001 | Prevent unintended engine acceleration | D | 100 ms |
| SG-ENG-002 | Prevent engine stall at high vehicle speed | B | 500 ms |
| SG-ENG-003 | Prevent fuel system failure causing fire | C | 200 ms |

**Fault Trees** (`FT-ENG-001`, `FT-ENG-002`) quantify the contribution of ThrottleControl, SafetyMonitor, and WatchdogTimer failures to the top hazard events.

**FMEA** tables (`FMEA-ENG-001`, `FMEA-ENG-002`) cover 16 failure modes across the throttle subsystem, fuel control, safety monitor, sensors, and hardware.

---

## Security analysis

`TARA-ENG-001` covers the CAN bus and OBD-II interface with 4 damage scenarios, 4 threat scenarios, 4 cybersecurity goals, and 4 security controls.

`VR-ENG-001` (CAN replay, **closed** — mitigated by AUTOSAR SecOC) and `VR-ENG-002` (firmware rollback via OBD-II, **open** — tracked) are the two VulnerabilityReport entries.

---

## Test cases

**Safety (ISO 26262)**

| ID | Title | Level |
|---|---|---|
| TC-ENG-SAFE-001 | HIL — safety system end-to-end fault response under all single-point faults | L5 |
| TC-ENG-SAFE-002 | HIL — TPS dual-track divergence triggers safe state within 50 ms | L5 |
| TC-ENG-SAFE-003 | HIL — hardware watchdog resets ECU within 50 ms of software lock-up | L5 |
| TC-ENG-SAFE-004 | Integration — SafetyMonitor detects all single-point faults within 100 ms | L4 |
| TC-ENG-SAFE-005 | HIL — rev limiter enforces fuel cut and ignition retard independently of TPS | L5 |
| TC-ENG-SAFE-006 | HIL — stuck-open throttle detected by position feedback verification | L5 |

**Security (ISO/SAE 21434)**

| ID | Title | Level |
|---|---|---|
| TC-ENG-SEC-001 | Integration — CAN security module rejects frames with invalid MAC | L2 |
| TC-ENG-SEC-002 | Integration — UDS programming session requires cryptographic challenge-response | L3 |
| TC-ENG-SEC-003 | Integration — firmware flash rejects image without valid ECDSA signature | L3 |
| TC-ENG-SEC-004 | Integration — readMemoryByAddress restricted to security access level 0x11/0x12 | L3 |

---

## Running the validator

```bash
syscribe -m model_auto/ validate
```

The model produces 0 errors and 1 intentional warning (`W803` on the open `VR-ENG-002` vulnerability report, which is by design — it is being tracked).
