---
id: REQ-TRS-SEC-002
type: Requirement
name: Cybersecurity Risk Determination + Untreated-Threat Gate (risk model, E845, W031, W032, cyber-risk)
title: Tool shall compute ISO/SAE 21434 risk per ThreatScenario, gate untreated high risk, and check CAL consistency
status: draft
reqDomain: software
verificationMethod: test
---

ISO/SAE 21434 requires, per `ThreatScenario`, a **risk determination** (§15.8)
from impact severity and attack feasibility, a **risk-treatment decision**
(§15.9 — `avoid`/`reduce`/`share`/`retain`), and management of any untreated
risk above the acceptance line (§9). Historically Syscribe stored
`attackFeasibility`, `damageSeverity` and `calLevel` but **never computed a risk
value**, offered no treatment field, and let an untreated high risk pass
validation silently. This is the security counterpart of the safety-mechanism
completeness gate (#17). This requirement adds the risk model, two fields, three
validation codes, and a read-only `cyber-risk` view (GH #30).

## Risk model (normative)

For a `ThreatScenario` the tool **shall** compute risk exactly as follows.

- **Severity rank** — `negligible`=0, `moderate`=1, `major`=2, `severe`=3. The
  tool **shall** take the **maximum** `damageSeverity` over the `DamageScenario`s
  named in the threat's `damageScenarios` (each resolved via the standard
  cross-reference resolver). If none resolve, or none carry a `damageSeverity`,
  the severity is **unknown**.
- **Feasibility rank** — `very_low`=0, `low`=1, `medium`=2, `high`=3, from the
  threat's own `attackFeasibility`. Missing/invalid → **unknown**.
- If either rank is **unknown**, the risk is **unknown**: the threat is listed
  but **not gated**.
- Otherwise `score = severity + feasibility` (range 0..6), mapped to a **level**:
  - `0–1` → `low`
  - `2–3` → `medium`
  - `4` → `high`
  - `5–6` → `critical`

There **shall** be exactly **one** risk-level definition shared by the validator
and the `cyber-risk` command.

## New fields on `ThreatScenario`

- **`riskTreatment`** — enum `avoid` | `reduce` | `share` | `retain`. An invalid
  value **shall** produce error **`E845`**.
- **`residualRisk`** — free-text note describing the risk remaining after
  treatment; **no** validation beyond parse.

## `E845` — invalid riskTreatment

The tool **shall** define error code **`E845`**, emitted when a
`ThreatScenario.riskTreatment` is present and is not one of `avoid`, `reduce`,
`share`, `retain`.

## `W031` — untreated high risk

The tool **shall** define warning code **`W031`**, emitted by `validate` for a
`ThreatScenario` whose computed risk level is `high` or `critical`, that has
**no** `riskTreatment`, **and** is **not addressed** by any `CybersecurityGoal`
(no `CybersecurityGoal.threatScenarios` lists it). The message **shall** name the
computed level.

`W031` is a **warning** (exit code unchanged), gateable via `--deny W031`, and
promotable to error via a `[profiles]` policy (#18). It is a warning — not an
error — to keep bundled-model exit codes at 0 and to match the codebase
convention that completeness gaps (W306/W029/W030) are warnings.

## `W032` — CAL inconsistency

The tool **shall** define warning code **`W032`**, emitted by `validate` for each
`CybersecurityGoal`: compute the maximum risk level over the `ThreatScenario`s it
lists; the **expected minimum CAL** is `low`→CAL1, `medium`→CAL2, `high`→CAL3,
`critical`→CAL4. If the goal's `calLevel` rank is **below** the expected rank, the
tool **shall** emit `W032` naming actual vs expected. `W032` fires only when at
least one linked threat has a **computable** risk. Like `W031`, it is a gateable
and profile-promotable warning.

## `cyber-risk` command

The tool **shall** provide a read-only command
`syscribe -m <root> cyber-risk [--json]` that lists each `ThreatScenario` with:
its `severity`, `feasibility`, computed `risk` level, `riskTreatment` (or `—`),
addressed-by-goal (yes/no), and a `flag` (`untreated` when it would trip W031,
`unknown` when risk is not computable, else `ok`).

- **text** mode is a Markdown table.
- **`--json`** mode is a JSON array of
  `{id, severity, feasibility, risk, treatment, addressed, flag}`.
- An empty/none result emits a notice and exits `0`.

The command is read-only, traversing frontmatter + the resolver, and shares the
single risk-level definition with the validator.

**Source:** ISO/SAE 21434 §9 / §15.8 / §15.9 cybersecurity audit; GH issue #30.
Companion to REQ-TRS-SEC-001 (safety↔security co-engineering) and the safety
completeness gate (#17). The warning-not-error decision is deliberate and
promotable via profiles (#18).

**Acceptance criteria:** a `severe`-impact + `high`-feasibility `ThreatScenario`
with no `riskTreatment` and no addressing `CybersecurityGoal` produces exactly
one `W031` naming the level; adding `riskTreatment` **or** an addressing
`CybersecurityGoal` clears the `W031`; a `ThreatScenario` with unknown severity or
feasibility produces no `W031`; a `CybersecurityGoal` whose `calLevel` is below
the expected CAL for its threats' max risk produces `W032`; an invalid
`riskTreatment` produces `E845`; the W031/W032 model `validate`s with no errors;
`validate --deny W031` exits non-zero in the presence of a `W031`; `cyber-risk`
text shows a risk level and the `untreated` flag; `cyber-risk --json` is valid
JSON carrying `id`, `severity`, `feasibility`, `risk`, `treatment`, `addressed`,
`flag`; an empty model exits `0`.
