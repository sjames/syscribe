# SIL 4 Computer-Based Interlocking — IEC 61508 / EN 50128

`EXAMPLE · REFERENCE MODEL · model_sil/`

A SIL 4 Computer-Based Interlocking (CBI) reference model for a mainline passenger railway station. The model spans system architecture, hardware decomposition, vital software components, deployment configuration, formal safety analysis under IEC 61508 / EN 50129 / EN 50128, and cybersecurity analysis under ISO/SAE 21434. It is the most demanding reference model in the repository, targeting the highest integrity level applicable to railway signalling.

---

## Model scope

The model covers a **single CBI** at a two-platform station with a facing junction. The interlocking controls and supervises:

- Two home signals (one per direction)
- Two points machines (facing and trailing junction)
- Three track circuit sections (approach, Platform 1, Platform 2)
- One level crossing with automatically operated barriers

The CBI communicates with the signaller workstation over a management LAN, with trackside object controllers over an EN 50159 Category 2 field bus, and between its two processor channels over a dedicated cross-comparison Ethernet bus.

Standards applied:

- **IEC 61508:2010** — Functional safety of E/E/PE systems; risk graph parameters for SIL determination
- **EN 50129:2018** — Railway safety-related electronic systems; safety case structure
- **EN 50128:2011+A1:2020** — Railway software; SIL 4 V-cycle, formal methods, diverse programming
- **EN 50159:2010** — Railway safety communication; Category 2 (open transmission system, adversarial threat)
- **ISO/SAE 21434:2021** — Cybersecurity engineering (CAL 3)
- **NIS2 Directive (EU 2022/2555)** — Critical infrastructure cybersecurity obligations

## Architecture

```
InterlockingSystem (hardware PartDef)
└── InterlockingControlSoftware (software PartDef, isDeploymentPackage)
    │   ── vital partition (SIL 4, MPU-protected) ──────────────
    ├── VitalSoftwareBase (abstract, silLevel: 4)
    │   ├── RouteProcessor       ← route set/lock/release logic
    │   ├── ConflictChecker      ← formally-verified conflict matrix
    │   ├── PointsController     ← position command and detection
    │   ├── SignalController     ← aspect authority, 5-condition check
    │   └── SafetyCommLayer      ← EN 50159 Cat 2 framing
    │   ── non-vital partition ──────────────────────────────────
    └── DiagnosticMonitor        ← read-only health reporting

Deployed as: Deployment::Station1Interlocking
  ├── vitalProcessorA  ╲  2oo2D cross-comparison
  ├── vitalProcessorB  ╱  via VitalSafetyLink
  ├── objectController1  (home signals)
  ├── objectController2  (points + level crossing)
  ├── trackCircuit1/2/3
  └── softwareImage (7 SWC instances)
```

The **2oo2D architecture** (two-out-of-two with diagnostics) means both processor channels independently compute every interlocking output. Any disagreement triggers an immediate fail-safe state — all signals to most-restrictive, all route locks preserved. This achieves the SIL 4 dangerous-failure-rate target of < 10⁻⁸ /h.

---

## Hardware

| Element | Type | Description |
|---|---|---|
| `System::InterlockingSystem` | PartDef (hardware) | Top-level CBI hardware; `processorChannels`, `fieldBusBaudRate` |
| `System::Hardware::VitalProcessor` | PartDef (abstract, SIL 4) | 2oo2D vital processor channel; `cpuFreqMhz`, `ramKb`, `flashKb` |
| `System::Hardware::ObjectController` | PartDef | Field object controller; drives signal heads, points machines, level crossing equipment |
| `System::Hardware::TrackCircuitInterface` | PartDef | Audio-frequency track circuit interface; reports occupancy to vital processor |
| `System::Hardware::SignalOutputModule` | PartDef | Signal aspect output; fail-safe to most-restrictive on power loss |
| `System::Hardware::PointsDriveModule` | PartDef | Points machine drive; detection circuit independent of drive circuit |
| `System::Hardware::LevelCrossingModule` | PartDef | Barrier control module; `barrierCount`, `warningTimeS` |

## Software components

| Element | SIL | Description |
|---|---|---|
| `System::Software::VitalSoftwareBase` | 4 (abstract) | Abstract base for all vital SWCs; cyclic execution ≤ 20 ms, B-Method specification obligation |
| `System::Software::RouteProcessor` | 4 | Evaluates and locks route requests; checks section clear, no conflict, points confirmed, LX confirmed |
| `System::Software::ConflictChecker` | 4 | Formally-verified conflict matrix; per-cycle CRC check; read-only flash |
| `System::Software::PointsController` | 4 | Commands point machines; waits for positively-proved detection; continuous supervision |
| `System::Software::SignalController` | 4 | Clears signals only when all five conditions true simultaneously; returns to danger within one scan on any failure |
| `System::Software::SafetyCommLayer` | 4 | EN 50159 Cat 2 framing: 32-bit CRC, sequence numbers, timestamp, source/destination address |
| `System::Software::DiagnosticMonitor` | non-vital | Read-only health reporting; MPU-enforced boundary prevents writes to vital partition |

---

## Requirements

### Safety (IEC 61508 / EN 50128)

| ID | Title | SIL |
|---|---|---|
| REQ-SIL-SAFE-001 | Conflict checker shall prevent any two conflicting routes from being simultaneously set | 4 |
| REQ-SIL-SAFE-002 | Signal controller shall only clear a signal when all interlocking conditions are simultaneously verified | 4 |
| REQ-SIL-SAFE-003 | Points controller shall verify detected position before reporting confirmation | 4 |
| REQ-SIL-SAFE-004 | Signal clearance shall be conditional on level crossing barriers confirmed lowered | 3 |

### Architecture and software process

| ID | Title | SIL |
|---|---|---|
| REQ-SIL-SW-001 | CBI vital processing shall use 2oo2D voting architecture with diverse processor channels | 4 |
| REQ-SIL-SW-002 | Vital software shall be developed by two independent diverse teams | 4 |
| REQ-SIL-SW-003 | Vital logic shall be formally specified using B-Method with machine-checked invariant proofs | 4 |
| REQ-SIL-SW-004 | Safety communication shall implement EN 50159 Category 2 safety codes | 4 |

### Security

| ID | Title |
|---|---|
| REQ-SIL-SEC-001 | Operator commands shall be authenticated before reaching the vital processor (mTLS 1.3 + TPM 2.0) |
| REQ-SIL-SEC-002 | Field bus commands shall include EN 50159 safety codes preventing replay and insertion |

---

## Safety analysis

**HARA** (IEC 61508 risk graph) identifies four hazardous events against three safety goals. The two highest-rated events use risk parameters Cd/Fb/Pb/W3, yielding a SIL 4 target.

| ID | Title | Risk params | SIL | FTTI |
|---|---|---|---|---|
| SG-SIL-001 | Prevent conflicting train routes set simultaneously | — | 4 | 50 ms |
| SG-SIL-002 | Prevent signal clearance unless all route conditions verified | — | 4 | 50 ms |
| SG-SIL-003 | Prevent signal clearance while level crossing barriers unconfirmed | — | 3 | 500 ms |
| HE-SIL-001 | Signal cleared into occupied track section | Cd/Fb/Pb/W3 | → 4 | — |
| HE-SIL-002 | Conflicting routes set simultaneously | Cd/Fb/Pb/W3 | → 4 | — |
| HE-SIL-003 | Signal cleared with points not in proved position | Cc/Fb/Pb/W3 | → 4 | — |
| HE-SIL-004 | Signal cleared while level crossing barriers not confirmed | Cc/Fb/Pa/W2 | → 3 | — |

**Fault Trees** demonstrate that the 2oo2D AND gate achieves the SIL 4 quantitative target:

| ID | Top event | Target | Result |
|---|---|---|---|
| FT-SIL-001 | Conflicting route set without detection | < 10⁻⁸ /h | ~2.5 × 10⁻⁹ /h ✓ |
| FT-SIL-002 | Signal cleared without all conditions satisfied | < 10⁻⁸ /h | ~4.0 × 10⁻⁹ /h ✓ |

**FMEA** tables cover 14 failure modes across the vital processor, safety communication subsystem, and trackside field equipment.

---

## Security analysis

`TARA-SIL-001` covers the operator workstation and field bus interfaces with 3 damage scenarios, 3 threat scenarios, 2 cybersecurity goals, and 3 security controls:

| Threat | Feasibility | Control |
|---|---|---|
| Compromised maintainer workstation issuing unauthorised route commands (TS-SIL-001) | medium | mTLS 1.3 with TPM-bound certificates (SC-SIL-001) |
| Replay of captured field bus point-movement command (TS-SIL-002) | low | EN 50159 Cat 2 sequence numbers (SC-SIL-002) |
| DoS flood on cross-comparison bus disrupting 2oo2D synchronisation (TS-SIL-003) | medium | VLAN rate-limiting on cross-comparison bus (SC-SIL-003) |

---

## Test cases

**Safety — SIL 4 interlocking (EN 50128)**

| ID | Title | Level |
|---|---|---|
| TC-SIL-SAFE-001 | HIL — 2oo2D channel disagreement forces immediate safe state | L5 |
| TC-SIL-SAFE-002 | HIL — signal reverts to red within one scan on section occupation | L5 |
| TC-SIL-SAFE-003 | Integration — conflict checker blocks simultaneous conflicting routes | L4 |
| TC-SIL-SAFE-004 | Integration — level crossing barriers must be confirmed before signal clearance | L4 |
| TC-SIL-SAFE-005 | Integration — PointsController position confirmation blocks signal clearance | L4 |

**Architecture and software process**

| ID | Title | Level |
|---|---|---|
| TC-SIL-SW-001 | Formal proof — ConflictChecker B-Method proofs and ProB model checking | L3 |
| TC-SIL-SW-002 | Integration — EN 50159 Cat 2 safety codes reject all defined fault classes | L4 |
| TC-SIL-SW-003 | Inspection — independent development process evidence for Channel A and B diversity | L2 |

**Security**

| ID | Title | Level |
|---|---|---|
| TC-SIL-SEC-001 | Integration — operator commands rejected without valid mTLS session | L4 |
| TC-SIL-SEC-002 | Integration — EN 50159 Cat 2 safety codes reject replay, insertion and timestamp violations | L4 |

**System integration**

| ID | Title | Level |
|---|---|---|
| TC-SIL-SYS-001 | HIL — end-to-end system integration across four scenarios (route setting, 2oo2D disagreement, TC transient, LX clearance) | L5 |

---

## Running the validator

```bash
syscribe -m model_sil/ validate
```

The model produces 0 errors and 0 warnings.
