---
id: REQ-TRS-FM-007
type: Requirement
name: Tool shall not report a mandatory feature as an always-selected optional feature (W012)
status: draft
reqDomain: software
verificationMethod: test
---

`groupKind:` groups a feature's **children** and `mandatory:` sets the feature's own **membership** ([[REQ-TRS-FM-004]]). A mandatory group whose children are optional is therefore written `mandatory: true` + `groupKind: optional`. Such a feature is meant to be in every product, so being selected in every `Configuration` is expected.

The tool **shall**:

- not raise `W012` for a `FeatureDef` that sets `mandatory: true`, whatever its `groupKind:`;
- keep raising `W012` for a `FeatureDef` with `groupKind: optional` and no `mandatory: true` that is selected in every `Configuration`, and name `mandatory: true` (not the legacy `groupKind: mandatory`) as the suggested spelling;
- keep raising `W011` for a feature selected in no `Configuration`, and call it an *optional* feature only when it is not mandatory.

**Source:** GH #326, found while building the grid-tie inverter model (`model_gti/`).

**Acceptance criteria:**

- A `FeatureDef` with `mandatory: true` and `groupKind: optional`, selected in every `Configuration`, raises no `W012`.
- An optional `FeatureDef` selected in every `Configuration` still raises `W012`.
- `model_gti/` spells its mandatory groups with optional children as `mandatory: true` + `groupKind: optional` and `feature-check` reports no findings.
