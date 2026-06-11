---
id: REQ-TRS-DISC-006
type: Requirement
title: Tool shall define warning W024 in feature-check for a FeatureDef that gates nothing and ships in nothing
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** define a new warning code **`W024`** for an **orphan feature**: a `FeatureDef` that is referenced by **no** element's `appliesWhen:` expression **and** is selected (`true`) by **no** `Configuration`. Such a feature gates nothing and ships in nothing — it is dead weight in the variability model.

`W024` **shall** be emitted by the `feature-check` command (per [[REQ-TRS-FM-001]]), **not** by the base `validate` command — base `validate` keeps its existing per-element behaviour unchanged.

A `FeatureDef` **shall not** be flagged with `W024` if **either**:

- it is selected (`true`) by at least one `Configuration` — e.g. a mandatory, group, or parent feature that ships in some product even though nothing is conditioned on it; **or**
- it is referenced as an operand by at least one element's `appliesWhen:` expression — i.e. it gates at least one element.

Only a feature failing **both** tests (gates nothing **and** ships in nothing) is an orphan.

`W024` **shall** be:

- a **warning** (not an error) — exit-code neutral by default, consistent with the `feature-check` exit contract in [[REQ-TRS-FM-001]];
- **gateable** via `--deny W024`, so a project may make orphan features fail the build.

`W023` is the previous highest assigned W-code (see [[REQ-TRS-IMPL-001]]); `W024` is therefore the next free slot in the sequence.

## Rationale

A feature that no element references and that no configuration selects contributes nothing to the product line but still appears in the feature model, in `features`/`feature` output, and in the Feature × Configuration grid — inflating the apparent variability and hiding modelling mistakes (a typo'd `appliesWhen:` operand, or a feature whose conditioning was never wired up). Surfacing it as an explicit, gateable warning lets product-line review catch dead features early, while the two exemptions ensure legitimately-shipping structural features (mandatory/group/parent) are never false-flagged.

**Source:** §9 (PLE); product-line feature discoverability; orphan analog of `W005` (orphan requirement) for the feature model, emitted by `feature-check` ([[REQ-TRS-FM-001]]).

**Acceptance criteria:** `feature-check` emits exactly one `W024` per `FeatureDef` that is referenced by no `appliesWhen:` and selected by no `Configuration`; a feature selected (`true`) by at least one `Configuration` is not flagged; a feature referenced by at least one element's `appliesWhen:` is not flagged; base `validate` never emits `W024`; `feature-check --deny W024` exits non-zero in the presence of a `W024`.
