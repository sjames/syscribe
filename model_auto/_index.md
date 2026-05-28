---
type: Package
name: EngineECU
---

Engine Control Unit (ECU) reference model for a passenger vehicle, demonstrating the full Syscribe format across system architecture, requirements traceability, functional safety analysis (ISO 26262), and cybersecurity analysis (ISO/SAE 21434).

## Model scope

The model covers the **Engine ECU** of a passenger car powertrain. The ECU controls the electronic throttle actuator and fuel injectors, reads crankshaft position and throttle position sensors, and communicates with the rest of the vehicle over a powertrain CAN bus. Safety monitoring is ASIL D, in line with ISO 26262 Hazardous Event classification of *unintended engine acceleration* and *engine stall at high speed*.

Standards applied:

- **ISO 26262:2018** — Functional safety for road vehicles; safety goals rated ASIL A–D
- **ISO/SAE 21434:2021** — Road vehicle cybersecurity engineering; CAL 1–3 security goals
- **AUTOSAR SecOC** — Message authentication on safety-critical CAN PDUs
- **UDS (ISO 14229)** — Diagnostic protocol with security access levels

## Architecture overview

```
EngineECU (hardware PartDef)
└── EngineControlSoftware (software PartDef, isDeploymentPackage)
    ├── SafetyMonitor (ASIL D)  ← supervises all others
    ├── ThrottleControl         ← PID, anti-windup, limp-home
    ├── FuelControl             ← lambda closed-loop, rev limiter
    ├── EngineStallMonitor      ← CPS tooth-count plausibility
    ├── CANSecurityModule       ← AUTOSAR SecOC MAC verify/generate
    ├── DiagnosticSecurityLayer ← UDS session / security access
    ├── SecureBootManager       ← ECDSA firmware signature verify
    └── BootSequence (ActionDef)

Deployed as: Vehicle::PowertrainECU
```

Sensors: `ThrottlePositionSensor` (dual-track TPS), `CrankshaftPositionSensor`, `LambdaSensor`  
Actuators: `ThrottleActuator` (H-bridge), `FuelInjector` (×N cylinders)  
Interfaces: `CANBusPort`, `CANBusConnection`, `OBDIIPort`, `PowertrainCANInterface`

---

## Directory

### System architecture

| Element | Type | Description |
|---|---|---|
| [System::EngineECU](System/EngineECU.md) | PartDef (hardware) | Top-level ECU hardware shell; supply voltage, operating temperature, CAN transceiver and watchdog features |
| [System::EngineControlSoftware](System/EngineControlSoftware.md) | PartDef (software, deploymentPackage) | Deployable software image; allocated to `System::EngineECU` |
| [System::Sensors::Sensor](System/Sensors/Sensor.md) | PartDef (abstract) | Abstract base for all sensors; `signalOutputType`, operating temperature range |
| [System::Sensors::ThrottlePositionSensor](System/Sensors/ThrottlePositionSensor.md) | PartDef | Dual-track resistive TPS; divergence threshold triggers ASIL D fault path |
| [System::Sensors::CrankshaftPositionSensor](System/Sensors/CrankshaftPositionSensor.md) | PartDef | Hall-effect CPS; tooth-count plausibility monitored by EngineStallMonitor |
| [System::Sensors::LambdaSensor](System/Sensors/LambdaSensor.md) | PartDef | Wideband lambda (UEGO); input to FuelControl closed-loop |
| [System::Actuators::ThrottleActuator](System/Actuators/ThrottleActuator.md) | PartDef | H-bridge driven throttle plate; spring-loaded fail-safe at 7 % |
| [System::Actuators::FuelInjector](System/Actuators/FuelInjector.md) | PartDef | Solenoid fuel injector; drive-circuit monitored by SafetyMonitor |
| [System::Hardware::CANTransceiver](System/Hardware/CANTransceiver.md) | PartDef | ISO 11898-2 high-speed CAN transceiver; `canIn` port |
| [System::Hardware::WatchdogTimer](System/Hardware/WatchdogTimer.md) | PartDef | Independent hardware watchdog; triggers ECU reset on software lock-up (REQ-ENG-SAFE-002) |

### Software components

| Element | Type | Description |
|---|---|---|
| [System::Software::SafetyMonitor](System/Software/SafetyMonitor.md) | PartDef (ASIL D) | Supervises all safety-relevant inputs and outputs; asserts fault signal to ThrottleControl and FuelControl; controlled shutdown within 200 ms |
| [System::Software::ThrottleControl](System/Software/ThrottleControl.md) | PartDef | PID position controller with anti-windup; limp-home mode at 7 %; `canOut` port |
| [System::Software::FuelControl](System/Software/FuelControl.md) | PartDef | Lambda closed-loop, rev limiter (soft retard at 6200 rpm, hard cut at 6500 rpm), fuel trim monitoring |
| [System::Software::EngineStallMonitor](System/Software/EngineStallMonitor.md) | PartDef | CPS tooth-count plausibility; DTC after 3 consecutive errors |
| [System::Software::CANSecurityModule](System/Software/CANSecurityModule.md) | PartDef | AUTOSAR SecOC MAC generation and verification on powertrain CAN PDUs (SC-ENG-001) |
| [System::Software::DiagnosticSecurityLayer](System/Software/DiagnosticSecurityLayer.md) | PartDef | UDS session management; security access level 0x11/0x12 with audit logging (SC-ENG-004) |
| [System::Software::SecureBootManager](System/Software/SecureBootManager.md) | PartDef | ECDSA P-256 firmware signature verification before flash programming (SC-ENG-003) |
| [System::Software::BootSequence](System/Software/BootSequence.md) | ActionDef | Ordered startup sequence: hardware init → secure boot → safety self-check → normal operation |

### Interfaces

| Element | Type | Description |
|---|---|---|
| [System::Interfaces::CANBusPort](System/Interfaces/CANBusPort.md) | PortDef | Typed endpoint for CAN connections |
| [System::Interfaces::CANBusConnection](System/Interfaces/CANBusConnection.md) | ConnectionDef | Typed CAN link; ends: `transmitter` and `receiver`, both `typedBy: CANBusPort` |
| [System::Interfaces::CANFrame](System/Interfaces/CANFrame.md) | ItemDef | CAN frame payload item; `arbitrationId`, `dlc`, `payload` |
| [System::Interfaces::OBDIIPort](System/Interfaces/OBDIIPort.md) | PortDef | OBD-II diagnostic interface port |
| [System::Interfaces::PowertrainCANInterface](System/Interfaces/PowertrainCANInterface.md) | InterfaceDef | Powertrain CAN bus interface definition |

### Deployment

| Element | Type | Description |
|---|---|---|
| [Vehicle::PowertrainECU](Vehicle/PowertrainECU.md) | Part | Concrete ECU instance for the vehicle; redefines `supplyVoltage`; contains all sub-part usages |
| [Allocations::SwToECU](Allocations/SwToECU.md) | Allocation | `EngineControlSoftware` allocated to `EngineECU`; documents MPU partitioning and calibration flash isolation |

### Requirements

| ID | Title | ASIL | Domain |
|---|---|---|---|
| [REQ-ENG-SYS-000](Requirements/REQ-ENG-SYS-000.md) | Engine ECU shall provide safe and efficient engine management | — | system |
| [REQ-ENG-SAFE-000](Requirements/Safety/REQ-ENG-SAFE-000.md) | Engine ECU shall prevent safety hazards identified in HARA | — | system |
| [REQ-ENG-SAFE-001](Requirements/Safety/REQ-ENG-SAFE-001.md) | Safety monitor shall detect all safety faults within 100 ms | D | software |
| [REQ-ENG-SAFE-002](Requirements/Safety/REQ-ENG-SAFE-002.md) | Hardware watchdog shall reset ECU within 50 ms of software lock-up | D | hardware |
| [REQ-ENG-SAFE-003](Requirements/Safety/REQ-ENG-SAFE-003.md) | TPS dual-track divergence shall trigger safe state within 50 ms | C | software |
| [REQ-ENG-SAFE-004](Requirements/Safety/REQ-ENG-SAFE-004.md) | Rev limiter shall enforce fuel cut independently of TPS | B | software |
| [REQ-ENG-SAFE-005](Requirements/Safety/REQ-ENG-SAFE-005.md) | Throttle close command shall be verified by TPS position feedback | D | software |
| [REQ-ENG-PERF-000](Requirements/Performance/REQ-ENG-PERF-000.md) | Engine ECU shall meet throttle response and fuel efficiency targets | — | system |
| [REQ-ENG-PERF-001](Requirements/Performance/REQ-ENG-PERF-001.md) | Throttle position shall reach commanded value within 150 ms | — | software |
| [REQ-ENG-PERF-002](Requirements/Performance/REQ-ENG-PERF-002.md) | Lambda closed-loop shall converge to target within 500 ms | — | software |
| [REQ-ENG-SEC-001](Requirements/Security/REQ-ENG-SEC-001.md) | Safety-critical CAN messages shall be authenticated using MAC | — | software |
| [REQ-ENG-SEC-002](Requirements/Security/REQ-ENG-SEC-002.md) | ECU calibration programming sessions shall require cryptographic authentication | — | software |
| [REQ-ENG-SEC-003](Requirements/Security/REQ-ENG-SEC-003.md) | Firmware updates shall require ECDSA P-256 signature verification | — | software |
| [REQ-ENG-SEC-004](Requirements/Security/REQ-ENG-SEC-004.md) | Diagnostic memory read access shall be restricted to authorised sessions | — | software |

### Architecture Decision Records

| ID | Title | Status |
|---|---|---|
| [ADR-ENG-SYS-001](Decisions/ADR-ENG-SYS-001.md) | Decompose system requirements into performance, safety, and security sub-trees | accepted |
| [ADR-ENG-SAFE-001](Decisions/ADR-ENG-SAFE-001.md) | ASIL D decomposition for engine safety requirement into SW + HW sub-requirements | accepted |
| [ADR-ENG-PERF-001](Decisions/ADR-ENG-PERF-001.md) | Decompose performance requirement into throttle response and fuel efficiency | accepted |

### Safety analysis

**Hazard Analysis and Risk Assessment (ISO 26262)**

| ID | Title | ASIL | FTTI |
|---|---|---|---|
| [SG-ENG-001](Safety/HARA/SG-ENG-001.md) | Prevent unintended engine acceleration | D | 100 ms |
| [SG-ENG-002](Safety/HARA/SG-ENG-002.md) | Prevent engine stall at high vehicle speed | B | 500 ms |
| [SG-ENG-003](Safety/HARA/SG-ENG-003.md) | Prevent fuel system failure causing fire | C | 200 ms |
| [HE-ENG-001](Safety/HARA/HE-ENG-001.md) | Throttle stuck open at >20 % during normal driving | — | — |
| [HE-ENG-002](Safety/HARA/HE-ENG-002.md) | Engine stall at highway speed | — | — |
| [HE-ENG-003](Safety/HARA/HE-ENG-003.md) | Fuel injector stuck open causing fire risk | — | — |

**Fault Tree Analysis**

| ID | Title | Top event |
|---|---|---|
| [FT-ENG-001](Safety/FTA/FT-ENG-001.md) | Fault tree — unintended engine acceleration | SG-ENG-001 |
| [FT-ENG-002](Safety/FTA/FT-ENG-002.md) | Fault tree — engine stall at high speed | SG-ENG-002 |

**Failure Mode and Effects Analysis**

| ID | Title | Entries |
|---|---|---|
| [FMEA-ENG-001](Safety/FMEA/FMEA-ENG-001.md) | FMEA — Throttle, Fuel Control and Safety Monitor Subsystem | FM-ENG-001 … FM-ENG-010 |
| [FMEA-ENG-002](Safety/FMEA/FMEA-ENG-002.md) | FMEA — Sensors and Hardware Subsystem | FM-ENG-011 … FM-ENG-016 |

### Security analysis

| ID | Title | Type |
|---|---|---|
| [TARA-ENG-001](Security/TARA-ENG-001.md) | TARA — Engine ECU CAN bus and OBD-II interface | TARASheet (4 DS · 4 TS · 4 CSGs · 4 SCs) |
| [VR-ENG-001](Security/VR-ENG-001.md) | CAN bus replay attack — mitigated by SecOC | VulnerabilityReport (closed) |
| [VR-ENG-002](Security/VR-ENG-002.md) | Firmware rollback via OBD-II exposes patched vulnerabilities | VulnerabilityReport (**open** — W803) |

### Test cases

**Safety (ISO 26262)**

| ID | Title | Level | Verifies |
|---|---|---|---|
| [TC-ENG-SAFE-001](Verification/TC-ENG-SAFE-001.md) | HIL — safety system end-to-end fault response under all single-point faults | L5 | REQ-ENG-SAFE-000 |
| [TC-ENG-SAFE-002](Verification/TC-ENG-SAFE-002.md) | HIL — TPS dual-track divergence triggers safe state within 50 ms | L5 | REQ-ENG-SAFE-003 |
| [TC-ENG-SAFE-003](Verification/TC-ENG-SAFE-003.md) | HIL — hardware watchdog resets ECU within 50 ms of software lock-up | L5 | REQ-ENG-SAFE-002 |
| [TC-ENG-SAFE-004](Verification/TC-ENG-SAFE-004.md) | Integration — SafetyMonitor detects all single-point faults within 100 ms | L4 | REQ-ENG-SAFE-001 |
| [TC-ENG-SAFE-005](Verification/TC-ENG-SAFE-005.md) | HIL — rev limiter enforces fuel cut and ignition retard independently of TPS | L5 | REQ-ENG-SAFE-004 |
| [TC-ENG-SAFE-006](Verification/TC-ENG-SAFE-006.md) | HIL — stuck-open throttle detected by position feedback verification | L5 | REQ-ENG-SAFE-005 |

**Performance**

| ID | Title | Level | Verifies |
|---|---|---|---|
| [TC-ENG-PERF-001](Verification/TC-ENG-PERF-001.md) | Integration — throttle step response within 150 ms | L3 | REQ-ENG-PERF-001 |
| [TC-ENG-PERF-002](Verification/TC-ENG-PERF-002.md) | Integration — lambda closed-loop convergence within 500 ms | L3 | REQ-ENG-PERF-002 |
| [TC-ENG-PERF-003](Verification/TC-ENG-PERF-003.md) | System integration — combined throttle and fuel control performance | L3 | REQ-ENG-PERF-000 |

**Security (ISO/SAE 21434)**

| ID | Title | Level | Verifies |
|---|---|---|---|
| [TC-ENG-SEC-001](Verification/TC-ENG-SEC-001.md) | Integration — CAN security module rejects frames with invalid MAC | L2 | REQ-ENG-SEC-001 |
| [TC-ENG-SEC-002](Verification/TC-ENG-SEC-002.md) | Integration — UDS programming session requires cryptographic challenge-response | L3 | REQ-ENG-SEC-002 |
| [TC-ENG-SEC-003](Verification/TC-ENG-SEC-003.md) | Integration — firmware flash rejects image without valid ECDSA signature | L3 | REQ-ENG-SEC-003 |
| [TC-ENG-SEC-004](Verification/TC-ENG-SEC-004.md) | Integration — readMemoryByAddress restricted to security access level 0x11/0x12 | L3 | REQ-ENG-SEC-004 |

**System integration**

| ID | Title | Level | Verifies |
|---|---|---|---|
| [TC-ENG-SYS-001](Verification/TC-ENG-SYS-001.md) | System integration — end-to-end engine management under nominal and fault conditions | L3 | REQ-ENG-SYS-000 |

### Diagrams

| Name | Kind | Subject |
|---|---|---|
| [System Overview](Diagrams/SystemOverview.md) | Mermaid | High-level block diagram of ECU components and relationships |
| [Sensor Hierarchy](Diagrams/SensorHierarchy.md) | SVG | Abstract `Sensor` base and concrete specialisations with inheritance arrows |
