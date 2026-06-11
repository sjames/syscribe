---
id: REQ-TRS-SAFE-005
type: Requirement
title: Tool shall compute and gate ISO 26262-5 §8-9 hardware architectural metrics per SafetyGoal
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** compute the quantitative hardware architectural metrics defined by
ISO 26262-5 §8–9 — Single-Point Fault Metric (**SPFM**), Latent-Fault Metric (**LFM**),
and Probabilistic Metric for random Hardware Failures (**PMHF**) — per `SafetyGoal`, gate
them against the goal's ASIL/SIL target, and expose them through a `metrics` command.

> **First-order approximation.** This is a first-order FMEDA-style roll-up that depends
> entirely on user-supplied diagnostic-coverage and failure-rate inputs. It is **not** a
> substitute for a full FMEDA and **must be independently verified** by the safety team
> before being used in a hardware safety case.

## Data model

A `SafetyGoal`'s **contributing hardware failures** are the `FaultTreeEvent`s that live
under the `FaultTree`(s) whose `topEvent` resolves to that goal. A `FaultTree`'s events
live in the subdirectory named after the tree (their qualified names are prefixed by the
tree's qualified name + `::`). `topEvent` → `SafetyGoal` resolution uses the `Resolver`.

Two optional fields are carried on `FaultTreeEvent` frontmatter:

| Field | YAML | Range | Meaning |
|---|---|---|---|
| `diagnostic_coverage` | `diagnosticCoverage` | `0.0`–`1.0` | DC — fraction of this event's failure rate detected by a safety mechanism |
| `latent_diagnostic_coverage` | `latentDiagnosticCoverage` | `0.0`–`1.0` | DCl — fraction of the *latent* part covered |

A value outside `0.0`–`1.0` for either field **shall** raise error **E846**.

## Metric formulas (exact)

For a `SafetyGoal`, over its contributing events that declare a `failureRate` (events with
no λ are skipped):

```
Σλ      = Σ λ_i
λ_RF    = Σ λ_i · (1 − DC_i)              DC_i defaults to 0 when absent
SPFM    = 1 − λ_RF / Σλ                   (Σλ = 0 → SPFM undefined / None)
λ_MPFL  = Σ λ_i · DC_i · (1 − DCl_i)      DCl_i defaults to 0 when absent
LFM     = 1 − λ_MPFL / (Σλ − λ_RF)        (None unless ≥1 event sets DCl; (Σλ−λ_RF)=0 → None)
PMHF    = λ_RF + λ_MPFL                    (/h)
```

`LFM` is computed **only** if at least one contributing event sets `latentDiagnosticCoverage`;
otherwise it is reported as `n/a`.

## Opt-in rule

A `SafetyGoal`'s metrics **shall** be computed and gated **only if at least one** of its
contributing `FaultTreeEvent`s declares `diagnosticCoverage`. If none do, the goal's metrics
are reported as `n/a (no diagnosticCoverage data)`, are not computed, and are **never gated**.
This keeps models without DC data silent (zero W033, unchanged exit codes).

## Targets

By the `SafetyGoal`'s `asilLevel` (ISO 26262):

| ASIL | SPFM ≥ | LFM ≥ | PMHF < (/h) |
|---|---|---|---|
| A | n/a | n/a | n/a |
| B | 0.90 | 0.60 | 1e-7 |
| C | 0.97 | 0.80 | 1e-7 |
| D | 0.99 | 0.90 | 1e-8 |

For a SIL-only goal (`silLevel`, IEC 61508), gate PMHF (as PFH):

| SIL | PMHF/PFH < (/h) |
|---|---|
| 2 | 1e-6 |
| 3 | 1e-7 |
| 4 | 1e-8 |

SPFM/LFM are reported for SIL goals but **not** gated.

## Validation rules

| Code | Severity | Condition |
|---|---|---|
| `E846` | error | `diagnosticCoverage` or `latentDiagnosticCoverage` outside `0.0`–`1.0` |
| `W033` | warning | A `SafetyGoal` with DC data has a computed SPFM, LFM, or PMHF that misses its ASIL/SIL target |

`W033` is a warning (per codebase convention so unannotated models stay at exit 0), is
gateable via `--deny W033`, and is profile-promotable. A single `W033` finding names which
metric(s) missed and reports actual vs target.

## `metrics` command

`syscribe -m <root> metrics [--json]` — read-only. One row per `SafetyGoal`: its ASIL/SIL,
SPFM, LFM, PMHF (or `n/a`), and overall pass/fail vs target. `--json` emits an array of
`{id, asil, sil, spfm, lfm, pmhf, pass}`; the default text output is a table. A goal with no
FaultTree or no DC data shows `n/a`.

## Acceptance criteria

(The fixture uses `SG-MET-001`/`SG-MET-002` as the pattern-valid `SG-*` ids for the worked
"SG-M-001"/"SG-M-002" goals; the numbers below are exact.)

- Goal **SG-MET-001** (ASIL D) with events λ=1.0e-7/DC=0.99 and λ=1.0e-7/DC=0.90 yields
  Σλ=2.0e-7, λ_RF=1.1e-8, **SPFM=0.945**, LFM=n/a, **PMHF=1.1e-8** ⇒ misses SPFM≥0.99 and
  PMHF<1e-8 ⇒ exactly one **W033** on SG-MET-001.
- Goal **SG-MET-002** (ASIL B) with event λ=1.0e-8/DC=0.99 yields Σλ=1e-8, λ_RF=1e-10,
  **SPFM=0.99**, **PMHF=1e-10** ⇒ passes ⇒ **no W033**.
- A `FaultTreeEvent` with `diagnosticCoverage: 1.5` raises **E846**.
- `metrics` text shows SG-M-001's SPFM and a fail verdict; `metrics --json` carries
  `spfm`/`pmhf`/`pass`.
- `--deny W033` exits non-zero when a goal misses its target.
- Bundled models with no `diagnosticCoverage` emit zero W033 and unchanged exit codes.
