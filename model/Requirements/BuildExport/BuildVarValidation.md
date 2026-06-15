---
type: Requirement
id: REQ-TRS-BUILD-030
name: "Validation diagnostics E050 and W050 for build variable consistency"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BUILD-000]
breakdownAdr: Decisions::BuildExportADR
tags:
  - validation
  - build-integration
---

The Syscribe validator shall enforce the following diagnostics in relation to build
variable declarations:

### E050 — Conflicting build variable values

When two or more features that are simultaneously selected in a `Configuration` declare
`buildExports:` entries with the same `var:` name but **different** `whenSelected:`
values, and the conflict is not resolved by a `buildOverrides:` entry of the same name
in that `Configuration`, the validator shall emit error **E050** identifying the
conflicting features and variable name. A model with an E050 error is ill-formed;
`build-config` shall abort before generating any output.

### W050 — Selected feature contributes no build variable

When a `Configuration` is validated and a selected `FeatureDef` has neither a
`buildExports:` field nor any `parameters:` entry with `buildVar:`, the validator
may emit warning **W050**.

W050 is **opt-in** (disabled by default). It is enabled by passing `--deny W050` on
the command line or by adding `warnAsError: [W050]` to the `[validation]` section of
`.syscribe.toml`. This allows models that do not use the build integration capability
to remain warning-free, while models that intend full build coverage can gate on it in CI.
