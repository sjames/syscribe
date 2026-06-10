---
id: REQ-TRS-SAFE-007
type: Requirement
name: Confirmation measures, assessment independence, and DIA/CIA responsibility
title: Tool shall track work-product responsibility (W038) and confirmation measures with independence (ConfirmationMeasure, W039)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** represent two artefacts that functional-safety and cybersecurity
assessors check early but which today live only in external prose, disconnected from
the model:

1. **who** is accountable for each work product (the DIA / CIA responsibility split,
   ISO 26262-8 §5 Development Interface Agreement, ISO/SAE 21434 §7 Cybersecurity
   Interface Agreement), and
2. the **confirmation measures** — confirmation reviews, functional-safety audit and
   assessment, cybersecurity assessment — with their required **independence level**
   (I1–I3), per ISO 26262-2 §6.

Both checks are **opt-in / dormant**: a model that has not adopted the respective
practice sees nothing and keeps its exit codes unchanged. This keeps the bundled
demo models (`model/`, `model_auto/`, `model_sil/`) clean.

## Part 1 — `responsibility` field and W038

The tool **shall** accept a new common frontmatter field `responsibility:` (string)
on any element (`RawFrontmatter.responsibility`). It names the accountable party or
organisation for the work product (the DIA / CIA split — e.g. `"OEM"`, `"Supplier-X"`).

The tool **shall** treat the following element types as **work products** for this check:

- native `Requirement`,
- `PartDef`, `Part`,
- `SafetyGoal`, `CybersecurityGoal`.

| Code | Severity | Condition |
|---|---|---|
| `W038` | warning | A **non-draft** work-product element declares no `responsibility:`. One finding per offending element, naming it. |

**Opt-in rule.** The W038 check is **dormant** unless **at least one** element in the
model declares `responsibility:`. A model where no element declares responsibility emits
zero W038 and keeps unchanged exit codes (projects that have not adopted DIA/CIA tracking
see nothing; bundled models stay clean). `W038` is a warning (codebase convention so
unannotated models stay at exit 0), gateable via `--deny W038`, and profile-promotable.

## Part 2 — `ConfirmationMeasure` type and W039

The tool **shall** recognise a new native element `type: ConfirmationMeasure`, with a
stable opaque id matching `^CM(-[A-Z0-9]{2,12})+-[0-9]{3,8}$` (`CM-*`). The id is a valid
cross-reference target and is added to `is_stable_id`. Fields:

- `measureType:` (string, `RawFrontmatter.measure_type`) ∈
  { `confirmation_review`, `functional_safety_audit`, `functional_safety_assessment`,
  `cybersecurity_assessment` }.
- `independenceLevel:` (string, `RawFrontmatter.independence_level`) ∈ { `I1`, `I2`, `I3` }.
- `status:` (existing) — lifecycle status.
- `confirms:` (string or list, `RawFrontmatter.confirms`) — the confirmed work-product
  ref(s); each resolves via the `Resolver` to a model element. (A list is used rather than
  reusing the single-valued `subject:` so one measure can confirm several work products.)

### Structural errors

| Code | Severity | Condition |
|---|---|---|
| `E847` | error | `ConfirmationMeasure` is missing `id`, `title`, or `status`. |
| `E848` | error | `id` does not match the `CM-*` pattern. |
| `E849` | error | `measureType` is not one of the four allowed values. |
| `E850` | error | `independenceLevel` is not one of `I1 · I2 · I3`. |
| `E851` | error | a `confirms:` ref does not resolve to any model element. |

### W039 — required independent assessment missing

| Code | Severity | Condition |
|---|---|---|
| `W039` | warning | A high-integrity item lacks its required independent assessment. Specifically: a `SafetyGoal` (or native `Requirement`) with `asilLevel: D` that is **not** the subject (`confirms:`) of any `ConfirmationMeasure` with `measureType: functional_safety_assessment` and `independenceLevel: I3`; and a `CybersecurityGoal` with `calLevel: CAL4` not confirmed by a `cybersecurity_assessment` at `I3`. The message names what is missing. |

**Opt-in rule.** The W039 check is **dormant** unless **at least one**
`ConfirmationMeasure` exists in the model. The flag therefore means "you do confirmation
tracking but this high item is missing its independent assessment" — bundled models, which
have no `ConfirmationMeasure`, stay silent. `W039` is a warning, gateable via `--deny W039`,
and profile-promotable.

### ASIL/CAL → independence mapping

The mapping is intentionally **minimal**: only `asilLevel: D → I3 functional_safety_assessment`
and `calLevel: CAL4 → I3 cybersecurity_assessment` are gated. Lower integrity levels
(ASIL A–C, CAL1–CAL3) are documented as future tightening and are **not** gated here.

## Acceptance criteria

- A non-draft work product (e.g. a `PartDef`) with no `responsibility:`, in a model where
  at least one other element declares `responsibility:`, yields exactly one **W038** naming
  the element; adding `responsibility:` to it clears the finding. Both models `validate` with
  no errors.
- A model where **no** element declares `responsibility:` emits **zero W038** (opt-in).
- A `SafetyGoal` with `asilLevel: D`, in a model that contains ≥1 `ConfirmationMeasure` but
  none that is an `I3 functional_safety_assessment` confirming that goal, yields **W039**;
  adding a `ConfirmationMeasure { measureType: functional_safety_assessment,
  independenceLevel: I3, confirms: SG-… }` clears it. Both models `validate` with no errors.
- A model with **no** `ConfirmationMeasure` emits **zero W039** (opt-in).
- A `ConfirmationMeasure` with an invalid `measureType` or `independenceLevel` yields the
  corresponding `E849`/`E850`.
- `--deny W038` / `--deny W039` exit non-zero when the respective finding is present.
- `syscribe spec types` lists `ConfirmationMeasure`; `syscribe ... template ConfirmationMeasure`
  prints a ready-to-fill skeleton.
</content>
</invoke>
