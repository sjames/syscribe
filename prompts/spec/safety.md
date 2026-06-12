# Syscribe Safety and Security Analysis Elements (§8.18)

Native element types for functional safety (ISO 26262, IEC 61508, ISO 13849-1) and
cybersecurity analysis (ISO/SAE 21434). Like `Requirement` and `TestCase` they carry
stable opaque IDs and are dispatched by the parser on `type:`.

---

## Tier 2 — HARA (Hazard Analysis and Risk Assessment)

### HazardousEvent — `HE-*`

A combination of a hazard and an operational situation.

```yaml
type: HazardousEvent
id: HE-BRAKE-001            # required; HE-*
name: "Unintended brake release at highway speed"
status: approved

# ISO 26262 risk parameters (use these OR the IEC 61508 set below)
severity: S3                # S0 · S1 · S2 · S3
exposure: E4                # E0 · E1 · E2 · E3 · E4
controllability: C2         # C0 · C1 · C2 · C3

# IEC 61508 risk graph parameters (alternative to ISO 26262 set above)
consequence: Cc             # Ca · Cb · Cc · Cd
freqExposure: Fb            # Fa · Fb
avoidance: Pa               # Pa · Pb
demandRate: W2              # W1 · W2 · W3
```

ASIL is derived automatically from S × E × C. IEC 61508 SIL is derived from the risk
graph. Both are informational; the integrity level on the `SafetyGoal` is the normative
value.

### SafetyGoal — `SG-*`

Top-level safety requirement derived from the HARA.

```yaml
type: SafetyGoal
id: SG-BRAKE-001            # required; SG-*
name: "Prevent unintended brake release"
status: approved
asilLevel: D                # A | B | C | D  — OR silLevel: 1–4 OR plLevel: a–e
safeState: "Hold last commanded brake pressure"   # description of the safe state
ftti: "20ms"                # fault-tolerant time interval (max time in a fault before the safe state)
hazardousEvents:            # required (W806 if absent)
  - HE-BRAKE-001
```

Validation: W801 (no integrity level), W806 (no hazardousEvents), W805 (no derived
requirements), E841 (derived Requirement missing integrity level).

---

## Tier 2 — TARA (Threat Analysis and Risk Assessment)

### DamageScenario — `DS-*`

```yaml
type: DamageScenario
id: DS-001
name: "Attacker gains control of steering"
status: approved
damageSeverity: severe      # severe · major · moderate · negligible
impactCategories:           # safety · financial · operational · privacy
  - safety
  - operational
hazardRef: SG-001           # optional: HazardousEvent/SafetyGoal this damage endangers
                            #   (string or list; safety↔security co-engineering)
```

A safety-tagged DamageScenario (`impactCategories` includes `safety`) with no `hazardRef` warns **W030**; a `hazardRef` that does not resolve to a `HazardousEvent`/`SafetyGoal` errors **E844**. Run `syscribe -m <root> co-analysis` for the hazard ⇄ threat overlap.

### ThreatScenario — `TS-*`

```yaml
type: ThreatScenario
id: TS-001
name: "CAN bus spoofing via OBD port"
status: approved
attackFeasibility: medium   # high · medium · low · very_low
attackVector: local         # network · adjacent · local · physical
damageScenarios: [DS-001]
hazardRef: SG-001           # optional: direct HazardousEvent/SafetyGoal link (string or list)
riskTreatment: reduce       # optional: avoid · reduce · share · retain (invalid → E845)
residualRisk: "Low after MAC on torque frames"   # optional free text
```

**Risk (ISO/SAE 21434 §15.8):** severity = max `damageSeverity` over `damageScenarios` (negligible=0…severe=3); feasibility from `attackFeasibility` (very_low=0…high=3); `score = severity + feasibility` → low (0–1), medium (2–3), high (4), critical (5–6); unknown if either is missing. A high/critical-risk threat with no `riskTreatment` and not listed by any `CybersecurityGoal.threatScenarios` warns **W031** (gateable `--deny W031`). View per-threat risk with `syscribe -m <root> cyber-risk`.

### CybersecurityGoal — `CSG-*`

```yaml
type: CybersecurityGoal
id: CSG-001
name: "Ensure authenticity of steering control commands"
status: approved
securityProperty: authenticity  # confidentiality · integrity · availability · authenticity
calLevel: CAL3              # CAL1 · CAL2 · CAL3 · CAL4
threatScenarios: [TS-001]
```

Validation: W802 (no implementing SecurityControl), W804 (no derived Requirement), W032 (`calLevel` below the expected minimum CAL for the max risk of its listed threats: low→CAL1, medium→CAL2, high→CAL3, critical→CAL4).

### SecurityControl — `SC-*`

```yaml
type: SecurityControl
id: SC-001
name: "HMAC authentication on CAN messages"
status: approved
controlType: prevention     # prevention · detection · response · recovery
implementsGoals: [CSG-001]
```

Architecture elements that realise a `SecurityControl` set `allocatedFrom: SC-001`
(accepts single string or list for multiple controls).

### VulnerabilityReport — `VR-*`

```yaml
type: VulnerabilityReport
id: VR-001
name: "CVE-2024-XXXX — OBD port CAN injection"
status: open                # open triggers W803
cvssScore: 7.5              # 0.0–10.0
cveId: CVE-2024-12345       # optional CVE identifier
mitigatedBy: [SC-001]
affectedElements:
  - UAV::Avionics::FlightController
```

### TARASheet — `TARA-*` (Option B container)

Single-file container whose section tables are exploded at parse time into individual
Tier 2 types. Use when a compact sheet is preferable to separate files.

```yaml
type: TARASheet
id: TARA-001
name: "TARA for braking system"
status: approved
damageTable:
  - id: DS-001
    name: "..."
    damageSeverity: severe
    impactCategories: [safety]
threatTable:
  - id: TS-001
    name: "..."
    attackFeasibility: medium
    attackVector: local
    damageScenarios: [DS-001]
goalTable:
  - id: CSG-001
    name: "..."
    securityProperty: authenticity
    calLevel: CAL3
controlTable:
  - id: SC-001
    name: "..."
    controlType: prevention
    implementsGoals: [CSG-001]
```

---

## Tier 4 — Fault Tree Analysis (FTA)

### Nesting rule (W900)

Gates and events **must** be placed in a subdirectory named after the FaultTree file:

```
Safety/FTA/FT-BRAKE-001.md             → Safety::FTA::FT-BRAKE-001
Safety/FTA/FT-BRAKE-001/
  FTG-BRAKE-001.md                     → Safety::FTA::FT-BRAKE-001::FTG-BRAKE-001
  FTE-BRAKE-001.md                     → Safety::FTA::FT-BRAKE-001::FTE-BRAKE-001
```

This is required so qualified names are prefixed by the tree's qualified name.

### FaultTree — `FT-*`

```yaml
type: FaultTree
id: FT-BRAKE-001
name: "Brake system fault tree"
status: approved
topEvent: SG-BRAKE-001      # required; must resolve to a SafetyGoal (E902)
```

### FaultTreeGate — `FTG-*`

```yaml
type: FaultTreeGate
id: FTG-BRAKE-001
name: "AND gate — dual failure"
gateType: AND               # AND · OR · XOR · NOT · inhibit
inputs:
  - FTG-BRAKE-001::FTE-BRAKE-001
  - FTG-BRAKE-001::FTE-BRAKE-002
```

Place in `FaultTreeName/` subdirectory.

### FaultTreeEvent — `FTE-*`

```yaml
type: FaultTreeEvent
id: FTE-BRAKE-001
name: "Hydraulic pump failure"
eventKind: basic            # basic · undeveloped · house
failureRate: 1.2e-7         # optional; per-hour failure rate (λ)
diagnosticCoverage: 0.99        # optional; DC, 0.0–1.0 (E846 if out of range)
latentDiagnosticCoverage: 0.90  # optional; DCl, 0.0–1.0 (E846 if out of range)
```

Place in `FaultTreeName/` subdirectory.

---

## Tier 4 — Quantitative HW safety metrics (ISO 26262-5 §8–9)

> **First-order FMEDA approximation** from your `failureRate` + diagnostic-coverage inputs — verify independently before use in a hardware safety case.

Per `SafetyGoal`, over the `FaultTreeEvent`s under the `FaultTree`(s) whose `topEvent` resolves to it (events with a `failureRate` only):

```
Σλ = Σ λ_i ; λ_RF = Σ λ_i·(1−DC_i) ; SPFM = 1 − λ_RF/Σλ
λ_MPFL = Σ λ_i·DC_i·(1−DCl_i) (events declaring DCl) ; LFM = 1 − λ_MPFL/(Σλ−λ_RF)
PMHF = λ_RF + λ_MPFL  (/h)
```

Targets — ASIL SPFM ≥ {B .90, C .97, D .99}; LFM ≥ {B .60, C .80, D .90}; PMHF < {B/C 1e-7, D 1e-8}/h. SIL gates PMHF/PFH < {SIL2 1e-6, SIL3 1e-7, SIL4 1e-8}/h only.

**Opt-in:** metrics are computed/gated only when at least one contributing event declares `diagnosticCoverage`; otherwise `n/a`. A missed target raises **W033** (gate `--deny W033`). Inspect with `syscribe -m model/ metrics [--json]`.

---

## Tier 4 — Freedom From Interference (ISO 26262-9 §7)

Two elements **share a resource** when both are `allocatedTo` (or, via `allocatedFrom` / an `Allocation` element, allocated to) the **same target**. Each element's integrity tag is `asilLevel`, else `silLevel` (→ `SIL<n>`), else `QM`. Two sources on one target with **differing** tags (including classified vs `QM`) are mixed-criticality and raise **W034** unless an FFI argument is present: the target **or** at least one source declares a non-empty `ffiRationale:` string, or carries a `breakdownAdr:` resolving to an `accepted` ADR.

```yaml
type: PartDef
name: HostECU
domain: hardware
ffiRationale: "MPU spatial partitioning + OS timing protection isolate the ASIL D and QM partitions (ISO 26262-6 §7.4.9)."
```

**Opt-in:** dormant unless some element declares `asilLevel`/`silLevel`. W034 is gateable (`--deny W034`).

---

## Confirmation measures & DIA/CIA responsibility (ISO 26262-2 §6 / -8 §5 / ISO/SAE 21434 §7)

**`responsibility:`** (common field, any element) — the accountable party/organisation
for a work product (the DIA/CIA split, e.g. `"OEM"` / `"Supplier-X"`).

**W038** — a **non-draft** work product (`Requirement`, `PartDef`, `Part`, `SafetyGoal`,
`CybersecurityGoal`) with no `responsibility:`. **Opt-in:** dormant unless some element
declares `responsibility:`. Gateable (`--deny W038`).

### ConfirmationMeasure — `CM-*`

```yaml
type: ConfirmationMeasure
id: CM-BRK-001
name: "Independent functional-safety assessment of the braking goal"
status: completed                 # planned | in_progress | completed
measureType: functional_safety_assessment
# confirmation_review | functional_safety_audit | functional_safety_assessment | cybersecurity_assessment  (else E849)
independenceLevel: I3             # I1 | I2 | I3  (else E850)
confirms:                         # work product ref(s); each must resolve (else E851)
  - SG-BRK-001
```

Errors: E847 (missing `id`/`name`/`status`), E848 (`id` not `CM-*`), E849/E850 (bad enum),
E851 (unresolved `confirms`).

**W039** — a high-integrity item lacking its required independent assessment: an `asilLevel: D`
`SafetyGoal`/native `Requirement` not confirmed by an I3 `functional_safety_assessment`; a
`calLevel: CAL4` `CybersecurityGoal` not confirmed by an I3 `cybersecurity_assessment`. (Lower
levels are future tightening, not gated.) **Opt-in:** dormant unless at least one
`ConfirmationMeasure` exists. Gateable (`--deny W039`).

---

## GSN safety-argument layer (§8.18)

The Goal Structuring Notation (GSN) argument layer. Render the tree with
`syscribe safety-case [<SG-id>] [--json]` — it also folds in the implicit
`SafetyGoal ← Requirement (derivedFromSafetyGoal) ← TestCase (verifies)` chain.

### Argument — `ARG-*`

```yaml
type: Argument
id: ARG-BRK-001
name: "Argue over independent torque monitoring"
status: approved
argumentType: strategy   # claim | strategy | solution   (absent → claim; else E854)
supports: SG-BRK-001     # SafetyGoal or parent Argument argued for (string or list; else E855)
evidence:                # Requirement / TestCase / sub-Argument / AssumptionOfUse (string or list; else E855)
  - REQ-BRK-001
  - TC-BRK-001
```

`id`/`name`/`status` required (else E852); id must match `ARG-*` (else E853).
**W040** — a `claim`/`strategy` Argument with empty `supports` AND empty `evidence`
(an orphan GSN node).

### AssumptionOfUse — `AOU-*`

```yaml
type: AssumptionOfUse
id: AOU-BRK-001
name: "Integrator provides a redundant torque sensor"
status: approved
appliesTo: SG-BRK-001    # SafetyGoal / Argument / Requirement it constrains (string or list; else E858)
```

`id`/`name`/`status` required (else E856); id must match `AOU-*` (else E857). A
safety-related application condition (SRAC).

---

## Tier 4 — FMEA

### FMEASheet — `FMEA-*`

```yaml
type: FMEASheet
id: FMEA-BRAKE-001
name: "Braking system FMEA"
status: approved
entries:
  - id: FM-001
    name: "Hydraulic line rupture"
    ref: UAV::Avionics::BrakingSystem   # optional; resolves to model element
    function: "Apply braking force"
    failureMode: "Loss of hydraulic pressure"
    failureEffect: "Vehicle cannot decelerate"
    fmeaSeverity: 9         # 1–10
    occurrence: 3           # 1–10
    detection: 4            # 1–10
    rpn: 108                # optional; auto-computed as S × O × D when absent
    recommendedAction: "Add redundant hydraulic line"   # required if RPN > 100
```

---

## Tier 4 — Attack path analysis (ISO/SAE 21434 §15.7)

Attack trees mirror FTA. An `AttackTree` substantiates a `ThreatScenario`
(`threatRef`) and decomposes it into `AttackTreeGate`s (`AND`/`OR`) and
`AttackStep` leaves. Feasibility rolls up **weakest-link**: rank
`very_low`=0 … `high`=3; an `AttackStep` = its `attackFeasibility` rank; an `AND`
gate (sequential path) = **MIN** of children; an `OR` gate (alternatives) =
**MAX** of children; the tree = the value of its single root child, mapped back
to a label. If the tree's computed feasibility differs from the linked
`ThreatScenario.attackFeasibility`, the validator emits **W035** (computed vs
declared).

### Nesting rule (W036)

Gates and steps **must** live in a subdirectory named after the AttackTree file
(checked by qualified-name prefix; an empty tree fires W036):

```
Security/Attacks/
  AT-TORQUE-001.md            ← AttackTree
  AT-TORQUE-001/              ← subdir named after the tree
    _index.md
    ATG-TORQUE-001.md         ← AttackTreeGate (root)
    ATG-TORQUE-002.md
    ATS-TORQUE-001.md         ← AttackStep
```

### AttackTree — `AT-*`

```yaml
type: AttackTree
id: AT-TORQUE-001
name: "Attack tree for TS-TORQUE-001"
status: approved
threatRef: TS-TORQUE-001     # required; must resolve to a ThreatScenario (E917)
```

### AttackTreeGate — `ATG-*`

```yaml
type: AttackTreeGate
id: ATG-TORQUE-001
name: "OR gate — bypass auth OR replay frame"
gateType: OR                 # AND (sequential path) | OR (alternatives) (E919)
inputs:                      # each must resolve to an ATG-*/ATS-* (E920)
  - ATG-TORQUE-002
  - ATS-TORQUE-001
```

Place in the `AttackTreeName/` subdirectory.

### AttackStep — `ATS-*`

```yaml
type: AttackStep
id: ATS-TORQUE-001
name: "Extract the bus authentication key"
attackFeasibility: high      # high | medium | low | very_low (E921)
```

Place in the `AttackTreeName/` subdirectory.

---

## Cross-domain integration rules

| Rule | Details |
|---|---|
| `Requirement` → `SafetyGoal` | `derivedFromSafetyGoal: SG-*`; integrity level must propagate (E841) |
| `Requirement` → `CybersecurityGoal` | `derivedFromSecurityGoal: CSG-*`; `verificationMethod:` required (W807) |
| `PartDef`/`Part` → `SecurityControl` | `allocatedFrom: SC-*` (or list); OSLC direction: arch element holds reference |
| `FaultTree` → `SafetyGoal` | `topEvent: SG-*` |
| `AttackTree` → `ThreatScenario` | `threatRef: TS-*` (E917); feasibility roll-up reconciled via W035 |
| `ASIL decomposition` | Lower level on derived element + `breakdownAdr:` (W808 without ADR) |
