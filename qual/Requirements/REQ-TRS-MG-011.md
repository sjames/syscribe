---
id: REQ-TRS-MG-011
type: Requirement
title: Configuration marked mg_variant is a parametric variant and may omit featureModel
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** let a `Configuration` opt in to being a MagicGrid **parametric variant**
via `custom_fields: { mg_variant: true }`, and for such a Configuration **shall relax the
`featureModel:` requirement**. This lets MagicGrid trade studies ([[REQ-TRS-MG-007]]) compare
solution alternatives that differ only by bound design parameters, without inventing a dummy
feature model — the friction hit while building `model_mg/`, where a throwaway `SystemFeatures`
model existed solely to satisfy `E201`. The existing `Configuration` element is reused (it is
the right concept and already owns `parameterBindings`); no new element type and no generic
`custom_fields` host are introduced.

### Relaxation (base format, scoped to the opt-in marker)

- `E201` currently requires `featureModel:` on every `Configuration`. It **shall not** require
  `featureModel:` when the Configuration sets `custom_fields: { mg_variant: true }`. `id`,
  `title`, and `status` remain required; the `id` still matches the `CONF-*` pattern (`E200`).
- A Configuration **without** the marker is unchanged — `featureModel:` stays mandatory. The
  relaxation is therefore inert for every non-MagicGrid model.

### Semantics of a feature-model-less variant

- A `Configuration` with `mg_variant: true` and no `featureModel:` denotes the **empty feature
  selection**: projection (§9) is the **identity** — every base element is active and nothing is
  feature-gated. `validate --config`, `matrix`, `diff`, and `trade-study` **shall** treat it as
  a normal configuration column whose only differentiator is its `parameterBindings`. No code
  path that consumes a `Configuration` may assume `featureModel:` is present (it must treat
  absent-with-`mg_variant` as the empty selection rather than erroring or panicking).
- `parameterBindings` drive the MoE evaluation of [[REQ-TRS-MG-007]] exactly as today.

### Gate check (active only under `--profile magicgrid`)

- **`MG070` — wrong host.** `custom_fields: { mg_variant: true }` on an element that is **not**
  a `Configuration`.

**Source:** MagicGrid parametric trade-study variants — friction identified building the
`model_mg/` EV-charging-station model (a dummy feature model was needed only to satisfy `E201`).
Reuses the `Configuration` element and its `parameterBindings`; feeds [[REQ-TRS-MG-007]].

**Acceptance criteria:**

- A `Configuration` with `custom_fields: { mg_variant: true }`, valid `CONF-*` `id`, `title`,
  `status`, and `parameterBindings`, but **no** `featureModel:`, validates with **no `E201`**.
- The same Configuration **without** the `mg_variant` marker still raises `E201` for the missing
  `featureModel:` (base behaviour unchanged).
- `trade-study` scores such a variant as a column from its `parameterBindings`; `validate --config`
  on it projects the identity (no gating) and does not error or panic.
- Under `validate --profile magicgrid`, `mg_variant: true` on a non-`Configuration` element raises
  `MG070`.
