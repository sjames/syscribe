# Safety & Security Analysis

`GUIDE · SAFETY & SECURITY ANALYSIS`

Syscribe provides first-class support for four standard safety and security analysis methods. Each maps to a dedicated set of element types with validated required fields, stable IDs, enum-checked parameters, and cross-reference rules enforced at validation time.

| Method | Standard | Element types | ID patterns |
|---|---|---|---|
| **HARA** | ISO 26262 / IEC 61508 / ISO 13849-1 | `HazardousEvent`, `SafetyGoal` | `HE-*`, `SG-*` |
| **TARA** | ISO/SAE 21434 | `TARASheet` → `DamageScenario`, `ThreatScenario`, `CybersecurityGoal`, `SecurityControl` | `TARA-*`, `DS-*`, `TS-*`, `CSG-*`, `SC-*` |
| **FTA** | IEC 61025 / ISO 26262-9 | `FaultTree`, `FaultTreeGate`, `FaultTreeEvent` | `FT-*`, `FTG-*`, `FTE-*` |
| **FMEA** | IEC 60812 / SAE J1739 | `FMEASheet` → `FMEAEntry` | `FMEA-*`, `FM-*` |

---

## Hazard and Risk Analysis (HARA)

### Integrity level standards

Syscribe supports three functional safety standards on the same `HazardousEvent` / `SafetyGoal` element pair. They differ only in the risk parameters used to derive the integrity level target.

| Standard | Domain | Risk input parameters | Integrity level field |
|---|---|---|---|
| **ISO 26262** | Automotive | `severity` S0–S3 · `exposure` E0–E4 · `controllability` C0–C3 | `asilLevel` A–D |
| **IEC 61508** | General E/E/ES | `consequence` Ca–Cd · `freqExposure` Fa/Fb · `avoidance` Pa/Pb · `demandRate` W1–W3 | `silLevel` 1–4 |
| **ISO 13849-1** | Machinery | — (architecture Category assessed separately) | `plLevel` a–e |

All parameters are optional and independent — use the set that matches your domain. W801 fires when a `SafetyGoal` has none of `asilLevel`, `silLevel`, or `plLevel` set.

### HazardousEvent

```yaml
---
type: HazardousEvent
id: HE-BRAKE-001
title: "Unintended brake release during high-speed cornering"
status: draft

# ISO 26262 — automotive
severity: S3          # S0 no injury | S1 light | S2 severe | S3 life-threatening
exposure: E3          # E0 incredibly unlikely … E4 high probability
controllability: C2   # C0 controllable | C1 simply | C2 normally | C3 uncontrollable
operationalSituation: "Vehicle traveling >80 km/h on curved road"

# IEC 61508 — non-automotive (use instead of S/E/C)
# consequence: Cc     # Ca slight | Cb serious | Cc death of one | Cd death of several
# freqExposure: Fb    # Fa rare/unlikely | Fb frequent/likely
# avoidance: Pb       # Pa possible | Pb barely possible
# demandRate: W3      # W1 very slight | W2 slight | W3 relatively high
---

Loss of brake hydraulic pressure due to solenoid valve failure allows
uncontrolled lateral drift at speed.
```

Required fields: `id` (HE-\* pattern), `title`, `status`. All risk parameters are optional but at least one set should be present for the analysis to be meaningful.

### SafetyGoal

```yaml
---
type: SafetyGoal
id: SG-BRAKE-001
title: "Prevent unintended brake release during vehicle motion"
status: draft

# ISO 26262
asilLevel: D

# IEC 61508 alternative
# silLevel: 3

# ISO 13849-1 alternative
# plLevel: e

safeState: "Both brake circuits remain engaged"
ftti: "50ms"
hazardousEvents:
  - HE-BRAKE-001
---

The brake system shall maintain hydraulic pressure on both axles at all
times during vehicle motion unless commanded by the driver.
```

`hazardousEvents:` must each resolve to a `HazardousEvent` element (E825). W801 fires if no integrity level field is set. W800 fires if a `HazardousEvent` is not referenced by any `SafetyGoal`.

### Generating templates

```bash
syscribe model/ template HazardousEvent > Safety/HARA/HE-BRAKE-001.md
syscribe model/ template SafetyGoal     > Safety/HARA/SG-BRAKE-001.md
```

---

## Threat Analysis and Risk Assessment (TARA)

The `TARASheet` is an **exploded container** (Option B). You write one file holding four section tables; the parser synthesises a first-class element for each row at parse time, so all cross-reference checks, graph edges, and validation rules apply automatically.

```yaml
---
type: TARASheet
id: TARA-SYS-001
title: "TARA — Vehicle ECU communication bus"
status: draft

damageTable:
  - id: DS-SYS-001
    title: "Unauthorized command injection causes vehicle manoeuvre"
    damageSeverity: severe
    impactCategories: [safety, operational]

threatTable:
  - id: TS-SYS-001
    title: "Attacker replays CAN frame via OBD-II port"
    attackFeasibility: medium
    attackVector: local
    damageScenarios: [DS-SYS-001]

goalTable:
  - id: CSG-SYS-001
    title: "Ensure integrity of safety-critical CAN messages"
    calLevel: CAL3
    securityProperty: integrity
    threatScenarios: [TS-SYS-001]

controlTable:
  - id: SC-SYS-001
    title: "Implement message authentication (MAC) on safety bus"
    controlType: prevention
    implementsGoals: [CSG-SYS-001]
---
```

Each synthesised element (`DS-SYS-001`, `TS-SYS-001`, etc.) is visible in the model browser and individually addressable by stable ID. Standalone element files (`DamageScenario`, `ThreatScenario`, etc.) are also accepted and follow the same validation rules.

### Generating a template

```bash
syscribe model/ template TARASheet > Safety/TARA-SYS-001.md
```

---

## Fault Tree Analysis (FTA)

FTA uses the **file-per-element** (Option A) pattern. Each node is its own `.md` file. The three element types map directly to standard FTA constructs.

### Directory layout

Gates and events must be **nested inside a subdirectory named after the FaultTree file**. The validator checks for children by qualified-name prefix (W900), so a flat layout where all files share the same parent directory will not satisfy the check.

```
Safety/FTA/
  FT-BRAKE-001.md               ← FaultTree
  FT-BRAKE-001/
    FTG-BRAKE-001.md            ← top-level OR gate
    FTG-BRAKE-002.md            ← sub-gate
    FTE-BRAKE-001.md            ← basic event
    FTE-BRAKE-002.md
```

This produces qualified names:

| File | Qualified name |
|---|---|
| `FT-BRAKE-001.md` | `Safety::FTA::FT-BRAKE-001` |
| `FT-BRAKE-001/FTG-BRAKE-001.md` | `Safety::FTA::FT-BRAKE-001::FTG-BRAKE-001` |
| `FT-BRAKE-001/FTE-BRAKE-001.md` | `Safety::FTA::FT-BRAKE-001::FTE-BRAKE-001` |

W900 fires if no `FaultTreeGate` or `FaultTreeEvent` element has a qualified name prefixed by the `FaultTree`'s own qname.

### FaultTree

```yaml
---
type: FaultTree
id: FT-BRAKE-001
title: "Fault tree — brake system loss of control"
status: draft
topEvent: Safety::SG-BRAKE-001   # must resolve to a SafetyGoal (E902)
# missionTime: "1e9 h"
---
```

### FaultTreeGate

```yaml
---
type: FaultTreeGate
id: FTG-BRAKE-001
title: "OR gate — hydraulic failure paths"
gateType: OR           # AND | OR | XOR | NOT | inhibit
inputs:
  - FTG-BRAKE-002      # child gate
  - FTE-BRAKE-001      # leaf event
---
```

`gateType` must be one of `AND`, `OR`, `XOR`, `NOT`, `inhibit` (E905). All `inputs:` must resolve to `FaultTreeGate` or `FaultTreeEvent` elements (E906). A gate with no inputs fires W901.

### FaultTreeEvent

```yaml
---
type: FaultTreeEvent
id: FTE-BRAKE-001
title: "Primary solenoid valve — total failure"
eventKind: basic          # basic | undeveloped | house
ref: Braking::SolenoidValve
failureRate: 1.0e-9       # failures per hour
---
```

`eventKind` values: `basic` (quantifiable leaf), `undeveloped` (not yet analysed), `house` (assumed event). The `ref:` field links the event to an architecture element.

### Generating templates

```bash
syscribe model/ template FaultTree      > Safety/FTA/FT-BRAKE-001.md
mkdir -p Safety/FTA/FT-BRAKE-001
syscribe model/ template FaultTreeGate  > Safety/FTA/FT-BRAKE-001/FTG-BRAKE-001.md
syscribe model/ template FaultTreeEvent > Safety/FTA/FT-BRAKE-001/FTE-BRAKE-001.md
```

---

## Failure Mode and Effects Analysis (FMEA)

FMEA uses the same **exploded container** pattern as TARA. One `FMEASheet` file; the parser synthesises a first-class `FMEAEntry` for each row.

```yaml
---
type: FMEASheet
id: FMEA-BRAKE-001
title: "FMEA — Brake Controller"
status: draft
entries:
  - id: FM-BRAKE-001
    ref: Braking::BrakeController
    failureMode: "Loss of output signal"
    effect: "No braking force applied"
    cause: "Software exception in control loop"
    fmeaSeverity: 9     # 1–10
    occurrence: 3       # 1–10
    detection: 4        # 1–10
    # rpn: 108          # auto-computed: 9 × 3 × 4 = 108
    recommendedAction: "Add independent watchdog monitor"
    satisfies: REQ-BRAKE-001
---
```

RPN is computed automatically from `fmeaSeverity × occurrence × detection` if all three are present. W903 fires when RPN > 100 and no `recommendedAction` is set.

### Generating a template

```bash
syscribe model/ template FMEASheet > Safety/FMEA-BRAKE-001.md
```

---

## Linking analysis to requirements

Safety and security analysis produces goals and findings that must be elaborated into traceable requirements. Two dedicated fields record the upstream motivation for each requirement.

### Safety requirements — `derivedFromSafetyGoal`

```yaml
type: Requirement
id: REQ-BRAKE-HYD-001
title: "Brake hydraulic pressure shall be maintained within 50 ms of loss detection"
reqDomain: software
status: draft
derivedFromSafetyGoal: SG-BRAKE-001   # the SafetyGoal that motivated this requirement
breakdownAdr: ADR-BRAKE-001
```

- **E832** — `derivedFromSafetyGoal` must resolve to a `SafetyGoal` element.
- **W805** — fires on a `SafetyGoal` that has no `Requirement` pointing back to it, indicating the goal has not yet been elaborated into a traceable requirement.

### Security requirements — `derivedFromSecurityGoal`

```yaml
type: Requirement
id: REQ-SEC-CAN-001
title: "ECU shall authenticate all CAN frames on the safety bus using CMAC"
reqDomain: software
status: draft
derivedFromSecurityGoal: CSG-SYS-001  # the CybersecurityGoal that motivated this requirement
breakdownAdr: ADR-SEC-CAN-001
```

- **E831** — `derivedFromSecurityGoal` must resolve to a `CybersecurityGoal` element.
- **W804** — fires on a `CybersecurityGoal` that has no `Requirement` pointing back to it.

---

## Development process chain

The full trace from a threat or hazard identification through to a verified implementation:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Safety (ISO 26262 / IEC 61508 / ISO 13849)                          │
│                                                                     │
│  HazardousEvent  →  SafetyGoal                                      │
│   (HE-*)             (SG-*)  ←────── derivedFromSafetyGoal ──────┐  │
│                         │                                         │  │
│                         └──── topEvent ──── FaultTree             │  │
│                                              (FT-*)               │  │
└─────────────────────────────────────────────────────────────────────┘
                                                                    │
┌─────────────────────────────────────────────────────────────────── │ ┐
│ Security (ISO/SAE 21434)                                           │  │
│                                                                    │  │
│  TARASheet → CybersecurityGoal ←── derivedFromSecurityGoal ───┐   │  │
│               (CSG-*)                                          │   │  │
│                  │ implementsGoals ←── SecurityControl         │   │  │
└─────────────────────────────────────────────────────────────────── │ ┘
                                                                    │  │
┌─────────────────────────────────────────────────────────────────── │ ─┘
│ Requirements & Architecture                                        │   │
│                                                                    │   │
│  Requirement (REQ-*) ─────────────────────────────────────────────┘   │
│   derivedFrom: parent REQ-*                                            │
│   breakdownAdr: ADR-*                                         ─────────┘
│        │
│        └── satisfies: ←── Part / PortDef / SecurityControl
│                                 │
│                                 └── allocatedTo: HW element
│
│  TestCase (TC-*)  verifies: REQ-*
└────────────────────────────────────────────────────────────────────────
```

### Traceability queries

```bash
# Show a safety goal and its integrity level
syscribe model/ show SG-BRAKE-001

# What requirements were derived from this goal?
syscribe model/ refs SG-BRAKE-001

# Full trace from a safety requirement upward and downward
syscribe model/ trace REQ-BRAKE-HYD-001

# What architecture elements satisfy a requirement?
syscribe model/ why REQ-BRAKE-HYD-001

# Which test cases cover a requirement?
syscribe model/ who-verifies REQ-BRAKE-HYD-001

# All elements that reference a cybersecurity goal
syscribe model/ refs CSG-SYS-001
```

### Validation rules summary

| Code | Severity | Description |
|---|---|---|
| E800 | Error | `HazardousEvent` missing required field (`id`, `title`, `status`) |
| E801–E803 | Error | ISO 26262 risk parameter out of valid range (S/E/C) |
| E804 | Error | HazardousEvent `id` does not match `HE-*` pattern |
| E805 | Error | `SafetyGoal` missing required field |
| E806 | Error | SafetyGoal `id` does not match `SG-*` pattern |
| E833–E836 | Error | IEC 61508 risk parameter out of valid range |
| E837 | Error | SafetyGoal `plLevel` not in `a`–`e` |
| E825 | Error | `hazardousEvents` ref does not resolve to a HazardousEvent |
| E831 | Error | `derivedFromSecurityGoal` does not resolve to a CybersecurityGoal |
| E832 | Error | `derivedFromSafetyGoal` does not resolve to a SafetyGoal |
| E902 | Error | `FaultTree.topEvent` does not resolve to a SafetyGoal |
| E906 | Error | `FaultTreeGate.inputs` ref is not a gate or event |
| W800 | Warning | HazardousEvent not referenced by any SafetyGoal |
| W801 | Warning | SafetyGoal has no integrity level (`asilLevel`, `silLevel`, or `plLevel`) |
| W804 | Warning | CybersecurityGoal has no `Requirement` with `derivedFromSecurityGoal` |
| W805 | Warning | SafetyGoal has no `Requirement` with `derivedFromSafetyGoal` |
