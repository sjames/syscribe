---
type: Package
name: Railway Interlocking System
---

SIL 4 Computer-Based Interlocking (CBI) reference model for a mainline passenger railway station. The model spans system architecture, hardware decomposition, vital software components, deployment configuration, formal safety analysis (IEC 61508 / EN 50129 / EN 50128), and cybersecurity analysis (ISO/SAE 21434 / NIS2) for a product certified to the highest integrity level applicable to railway signalling.

## Model scope

The model covers a single **Computer-Based Interlocking** at a two-platform station with a facing junction. The CBI controls and supervises:

- Two **home signals** (one per direction)
- Two **points machines** (facing and trailing junction)
- Three **track circuit sections** (approach, platform 1, platform 2)
- One **level crossing** with automatically operated barriers

The CBI communicates with the signaller workstation over a management LAN, with object controllers over an EN 50159 Category 2 field bus, and between its two processor channels over a dedicated cross-comparison Ethernet bus.

Standards applied:

- **IEC 61508:2010** — Functional safety of E/E/PE systems; risk graph parameters for SIL target determination
- **EN 50129:2018** — Railway safety-related electronic systems; safety case structure
- **EN 50128:2011+A1:2020** — Railway software; SIL 4 V-cycle, formal methods, diverse programming
- **EN 50159:2010** — Railway safety communication; Category 2 (open transmission system, adversarial threat)
- **ISO/SAE 21434:2021** — Road/rail vehicle cybersecurity engineering (CAL 3)
- **NIS2 Directive (EU 2022/2555)** — Critical infrastructure cybersecurity obligations

## Architecture overview

```
InterlockingSystem (hardware PartDef)
└── InterlockingControlSoftware (software PartDef, isDeploymentPackage)
    │   ── vital partition (SIL 4, MPU-protected) ──────────────────
    ├── VitalSoftwareBase (abstract, silLevel: 4)
    │   ├── RouteProcessor       ← route set/lock/release logic
    │   ├── ConflictChecker      ← formally-verified conflict matrix
    │   ├── PointsController     ← position command and detection
    │   ├── SignalController     ← aspect authority, 5-condition check
    │   └── SafetyCommLayer      ← EN 50159 Cat 2 framing on field bus
    │   ── non-vital partition ─────────────────────────────────────
    └── DiagnosticMonitor        ← read-only health reporting (non-vital)

RouteSettingSequence (ActionDef)  ← ordered signaller workflow

Deployed as: Deployment::Station1Interlocking
  ├── vitalProcessorA  ╲  2oo2D cross-comparison
  ├── vitalProcessorB  ╱  via VitalSafetyLink
  ├── objectController1  (home signals)
  ├── objectController2  (points + level crossing)
  ├── trackCircuit1/2/3
  └── softwareImage
      └── routeProcessor / conflictChecker / … (7 SWC instances)
```

The **2oo2D architecture** (two-out-of-two with diagnostics) means both channels independently compute the interlocking output. Any disagreement between channels triggers an immediate fail-safe state — all signals to most-restrictive aspect, all route locks released. This is the primary mechanism that achieves the SIL 4 dangerous-failure-rate target of < 10⁻⁸ /h.

---

## Directory

### System architecture — hardware

| Element | Type | Description |
|---|---|---|
| [System::InterlockingSystem](System/InterlockingSystem.md) | PartDef (hardware) | Top-level CBI hardware definition; `processorChannels`, `fieldBusBaudRate` |
| [System::Hardware::VitalProcessor](System/Hardware/VitalProcessor.md) | PartDef (abstract, SIL 4) | 2oo2D vital processor channel; `cpuFreqMhz`, `ramKb`, `flashKb`; satisfies REQ-SIL-SW-001, REQ-SIL-SEC-001 |
| [System::Hardware::ObjectController](System/Hardware/ObjectController.md) | PartDef | Field object controller; drives signal heads, points machines, level crossing equipment via local interfaces |
| [System::Hardware::TrackCircuitInterface](System/Hardware/TrackCircuitInterface.md) | PartDef | Audio-frequency track circuit interface; `sectionLength`; reports occupancy to vital processor |
| [System::Hardware::SignalOutputModule](System/Hardware/SignalOutputModule.md) | PartDef | Signal aspect output module; `aspectCount`; fail-safe to most-restrictive on power loss |
| [System::Hardware::PointsDriveModule](System/Hardware/PointsDriveModule.md) | PartDef | Points machine drive module; `strokeMs`, `detectVoltage`; detection circuit independent of drive circuit |
| [System::Hardware::LevelCrossingModule](System/Hardware/LevelCrossingModule.md) | PartDef | Level crossing barrier control module; `barrierCount`, `warningTimeS` |

### System architecture — software

| Element | Type | Description |
|---|---|---|
| [System::InterlockingControlSoftware](System/InterlockingControlSoftware.md) | PartDef (software, deploymentPackage) | Deployable software image; vital partition + non-vital partition; EN 50128 SIL 4 qualified |
| [System::Software::VitalSoftwareBase](System/Software/VitalSoftwareBase.md) | PartDef (abstract, SIL 4) | Abstract base for all vital SWCs; cyclic execution ≤ 20 ms, safe-side failure behaviour, B-Method specification obligation; satisfies REQ-SIL-SW-002, REQ-SIL-SW-003 |
| [System::Software::RouteProcessor](System/Software/RouteProcessor.md) | PartDef (SIL 4) | Evaluates and locks route requests; checks section clear, no conflict, points confirmed, LX confirmed; implements approach locking and overlap release |
| [System::Software::ConflictChecker](System/Software/ConflictChecker.md) | PartDef (SIL 4) | Maintains formally-verified conflict matrix; per-cycle CRC check; read-only flash storage; satisfies REQ-SIL-SAFE-001 |
| [System::Software::PointsController](System/Software/PointsController.md) | PartDef (SIL 4) | Commands point machines; waits for positively-proved detection current; `moveTimeoutMs`; continuous supervision during route occupancy; satisfies REQ-SIL-SAFE-003 |
| [System::Software::SignalController](System/Software/SignalController.md) | PartDef (SIL 4) | Clears signals only when all five conditions true simultaneously; returns to danger within one scan on any condition false; satisfies REQ-SIL-SAFE-002, REQ-SIL-SAFE-004 |
| [System::Software::SafetyCommLayer](System/Software/SafetyCommLayer.md) | PartDef (SIL 4) | EN 50159 Cat 2 framing on all field bus messages: 32-bit CRC, 32-bit sequence number, timestamp, source/destination address; satisfies REQ-SIL-SW-004, REQ-SIL-SEC-002 |
| [System::Software::DiagnosticMonitor](System/Software/DiagnosticMonitor.md) | PartDef (non-vital) | Read-only health and event reporting to maintainer workstation; MPU-enforced boundary prevents writes to vital partition |
| [System::Software::RouteSettingSequence](System/Software/RouteSettingSequence.md) | ActionDef | Ordered succession: `requestRoute` → `checkConflicts` → `lockPoints` → `lockRoute` → `clearSignal` |

### Interfaces

| Element | Type | Description |
|---|---|---|
| [System::Interfaces::VitalDataPort](System/Interfaces/VitalDataPort.md) | PortDef | Inter-channel port carrying EN 50159 Cat 2 framed output state vector between Channel A and Channel B |
| [System::Interfaces::FieldBusPort](System/Interfaces/FieldBusPort.md) | PortDef | Field bus port to object controllers; 1 Mbit/s industrial Ethernet, 20 ms deterministic polling cycle |
| [System::Interfaces::VitalSafetyLink](System/Interfaces/VitalSafetyLink.md) | ConnectionDef | Cross-comparison connection; ends: `channelA` and `channelB`, both `typedBy: VitalDataPort` |

### Deployment — Station 1 Interlocking

| Element | Type | Description |
|---|---|---|
| [Deployment::Station1Interlocking](Deployment/Station1Interlocking.md) | Part | Concrete CBI instance at Station 1; `typedBy: System::InterlockingSystem`; includes VitalSafetyLink connection between channels |
| Station1Interlocking::**vitalProcessorA** | Part | Channel A vital processor; primary channel for signaller workstation communication and clock master |
| Station1Interlocking::**vitalProcessorB** | Part | Channel B vital processor; diverse execution; cross-compares with Channel A every scan cycle |
| Station1Interlocking::**objectController1** | Part | Home Signals OC; drives Signal A (down) and Signal B (up) |
| Station1Interlocking::**objectController2** | Part | Points and LX OC; drives Points Machines 1 and 2, Level Crossing LC-001 |
| Station1Interlocking::**trackCircuit1/2/3** | Parts | Approach section, Platform 1, Platform 2 track circuit interfaces |
| Station1Interlocking::**softwareImage** | Part | `typedBy: InterlockingControlSoftware`; contains 7 SWC instances (`routeProcessor`, `conflictChecker`, `pointsController`, `signalController`, `safetyCommLayer`, `diagnosticMonitor`, `routeSetting`) |

### Allocations

| Element | Description |
|---|---|
| [Allocations::SwToVitalProcessor](Allocations/SwToVitalProcessor.md) | `InterlockingControlSoftware` → `VitalProcessor`; documents vital/non-vital partition isolation and commissioning parameter loading |
| [Allocations::SwToInterlocking](Allocations/SwToInterlocking.md) | `InterlockingControlSoftware` → `InterlockingSystem`; satisfies E314 `isDeploymentPackage` constraint |

### Requirements

**System**

| ID | Title | SIL | Domain |
|---|---|---|---|
| [REQ-SIL-SYS-000](Requirements/REQ-SIL-SYS-000.md) | Railway interlocking shall prevent unsafe train movements in all operating conditions | — | system |

**Safety (derived from HARA)**

| ID | Title | SIL | Verification |
|---|---|---|---|
| [REQ-SIL-SAFE-000](Requirements/Safety/REQ-SIL-SAFE-000.md) | CBI shall implement SIFs achieving SIL 4 for all identified hazards | — | — |
| [REQ-SIL-SAFE-001](Requirements/Safety/REQ-SIL-SAFE-001.md) | Conflict checker shall prevent any two conflicting routes from being simultaneously set | 4 | analysis |
| [REQ-SIL-SAFE-002](Requirements/Safety/REQ-SIL-SAFE-002.md) | Signal controller shall only clear a signal when all interlocking conditions are simultaneously verified | 4 | analysis |
| [REQ-SIL-SAFE-003](Requirements/Safety/REQ-SIL-SAFE-003.md) | Points controller shall verify detected position before reporting confirmation | 4 | test |
| [REQ-SIL-SAFE-004](Requirements/Safety/REQ-SIL-SAFE-004.md) | Signal clearance shall be conditional on level crossing barriers confirmed lowered | 3 | test |

**Architecture and implementation**

| ID | Title | SIL | Verification |
|---|---|---|---|
| [REQ-SIL-SW-001](Requirements/Software/REQ-SIL-SW-001.md) | CBI vital processing shall use 2oo2D voting architecture with diverse processor channels | 4 | inspection |
| [REQ-SIL-SW-002](Requirements/Software/REQ-SIL-SW-002.md) | Vital software shall be developed by two independent diverse teams | 4 | inspection |
| [REQ-SIL-SW-003](Requirements/Software/REQ-SIL-SW-003.md) | Vital logic shall be formally specified using B-Method with machine-checked invariant proofs | 4 | analysis |
| [REQ-SIL-SW-004](Requirements/Software/REQ-SIL-SW-004.md) | Safety communication shall implement EN 50159 Category 2 safety codes | 4 | test |

**Security**

| ID | Title | Domain |
|---|---|---|
| [REQ-SIL-SEC-001](Requirements/Security/REQ-SIL-SEC-001.md) | Operator commands shall be authenticated before reaching the vital processor (mTLS 1.3 + TPM) | system |
| [REQ-SIL-SEC-002](Requirements/Security/REQ-SIL-SEC-002.md) | Field bus commands shall include EN 50159 safety codes preventing replay and insertion | software |

### Architecture Decision Records

| ID | Title | Status |
|---|---|---|
| [ADR-SIL-SYS-001](ADRs/ADR-SIL-SYS-001.md) | Use 2oo2D voting architecture rather than 2oo3 for vital processing | accepted |
| [ADR-SIL-SW-001](ADRs/ADR-SIL-SW-001.md) | Use B-Method (Event-B) for formal specification of all vital interlocking logic | accepted |
| [ADR-SIL-COMM-001](ADRs/ADR-SIL-COMM-001.md) | Implement EN 50159 Category 2 safety codes on all vital communication paths | accepted |
| [ADR-SIL-ARCH-001](ADRs/ADR-SIL-ARCH-001.md) | Diverse software development teams for Channel A and Channel B vital logic | accepted |

### Safety analysis

**Hazard Analysis and Risk Assessment (IEC 61508 risk graph)**

| ID | Title | Consequence | Demand | SIL target | FTTI |
|---|---|---|---|---|---|
| [SG-SIL-001](Safety/HARA/SG-SIL-001.md) | Prevent conflicting train routes set simultaneously | — | — | 4 | 50 ms |
| [SG-SIL-002](Safety/HARA/SG-SIL-002.md) | Prevent signal clearance unless all route conditions verified | — | — | 4 | 50 ms |
| [SG-SIL-003](Safety/HARA/SG-SIL-003.md) | Prevent signal clearance while level crossing barriers unconfirmed | — | — | 3 | 500 ms |
| [HE-SIL-001](Safety/HARA/HE-SIL-001.md) | Signal cleared into occupied track section | Cd / Fb / Pb | W3 | → 4 | — |
| [HE-SIL-002](Safety/HARA/HE-SIL-002.md) | Conflicting routes set simultaneously | Cd / Fb / Pb | W3 | → 4 | — |
| [HE-SIL-003](Safety/HARA/HE-SIL-003.md) | Signal cleared with points not in proved position | Cc / Fb / Pb | W3 | → 4 | — |
| [HE-SIL-004](Safety/HARA/HE-SIL-004.md) | Signal cleared while level crossing barriers not confirmed down | Cc / Fb / Pa | W2 | → 3 | — |

**Fault Tree Analysis** (mission time 350 400 h / 40 years)

| ID | Title | Top event | Target | Result |
|---|---|---|---|---|
| [FT-SIL-001](Safety/FTA/FT-SIL-001.md) | Conflicting route set without detection | SG-SIL-001 | < 10⁻⁸ /h | ~2.5 × 10⁻⁹ /h ✓ |
| [FT-SIL-002](Safety/FTA/FT-SIL-002.md) | Signal cleared without all conditions satisfied | SG-SIL-002 | < 10⁻⁸ /h | ~4.0 × 10⁻⁹ /h ✓ |

**Failure Mode and Effects Analysis**

| ID | Title | Entries |
|---|---|---|
| [FMEA-SIL-001](Safety/FMEA/FMEA-SIL-001.md) | FMEA — Vital Processor and Safety Communication | FM-SIL-001 … FM-SIL-008 |
| [FMEA-SIL-002](Safety/FMEA/FMEA-SIL-002.md) | FMEA — Trackside Field Equipment | FM-SIL-009 … FM-SIL-014 |

### Security analysis

| ID | Title | Type |
|---|---|---|
| [TARA-SIL-001](Security/TARA-SIL-001.md) | TARA — Railway CBI operator workstation and field bus interfaces | TARASheet (3 DS · 3 TS · 2 CSGs · 3 SCs) |

Key threat scenarios: compromised maintainer workstation issuing unauthorised route commands (TS-SIL-001, `medium` feasibility); replay of captured field bus point-movement command (TS-SIL-002, `low`); DoS flood on cross-comparison bus (TS-SIL-003, `medium`). Security controls: mTLS 1.3 with TPM-bound certificates (SC-SIL-001); EN 50159 Cat 2 on field bus (SC-SIL-002); VLAN rate-limiting on cross-comparison bus (SC-SIL-003).

### Test cases

**Safety — SIL 4 interlocking (EN 50128)**

| ID | Title | Level | Verifies |
|---|---|---|---|
| [TC-SIL-SAFE-001](Verification/TC-SIL-SAFE-001.md) | HIL — 2oo2D channel disagreement forces immediate safe state | L5 | REQ-SIL-SAFE-000 |
| [TC-SIL-SAFE-002](Verification/TC-SIL-SAFE-002.md) | HIL — signal reverts to red within one scan cycle on section occupation | L5 | REQ-SIL-SAFE-002 |
| [TC-SIL-SAFE-003](Verification/TC-SIL-SAFE-003.md) | Integration — conflict checker blocks simultaneous conflicting route requests | L4 | REQ-SIL-SAFE-001 |
| [TC-SIL-SAFE-004](Verification/TC-SIL-SAFE-004.md) | Integration — level crossing barriers must be confirmed before signal clearance | L4 | REQ-SIL-SAFE-004 |
| [TC-SIL-SAFE-005](Verification/TC-SIL-SAFE-005.md) | Integration — PointsController position confirmation blocks signal clearance | L4 | REQ-SIL-SAFE-003 |

**Architecture and software process**

| ID | Title | Level | Verifies |
|---|---|---|---|
| [TC-SIL-SW-001](Verification/TC-SIL-SW-001.md) | Formal proof — ConflictChecker conflict matrix B-Method proofs and ProB model checking | L3 | REQ-SIL-SW-003 |
| [TC-SIL-SW-002](Verification/TC-SIL-SW-002.md) | Integration — EN 50159 Category 2 safety codes reject all defined fault classes | L4 | REQ-SIL-SW-004 |
| [TC-SIL-SW-003](Verification/TC-SIL-SW-003.md) | Inspection — independent development process evidence for Channel A and B diversity | L2 | REQ-SIL-SW-002 |

**Security**

| ID | Title | Level | Verifies |
|---|---|---|---|
| [TC-SIL-SEC-001](Verification/TC-SIL-SEC-001.md) | Integration — operator commands rejected without valid mTLS session | L4 | REQ-SIL-SEC-001 |
| [TC-SIL-SEC-002](Verification/TC-SIL-SEC-002.md) | Integration — EN 50159 Cat 2 safety codes reject replay, insertion and timestamp violations | L4 | REQ-SIL-SEC-002 |

**System integration**

| ID | Title | Level | Verifies |
|---|---|---|---|
| [TC-SIL-SYS-001](Verification/TC-SIL-SYS-001.md) | HIL — end-to-end system integration for SIL 4 interlocking safe state across four scenarios | L5 | REQ-SIL-SYS-000 |

### Diagrams

| Name | Kind | Subject |
|---|---|---|
| [Interlocking Architecture](Diagrams/InterlockingArchitecture.md) | Mermaid | 2oo2D vital processor pair, Object Controllers, field equipment and level crossing at Station 1 |
