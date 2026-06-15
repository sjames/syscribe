---
type: Requirement
id: REQ-TRS-BUILD-017
name: "build-config shall validate the configuration via SAT before generating output"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BUILD-010]
breakdownAdr: Decisions::BuildExportADR
tags:
  - cli
  - validation
  - build-integration
---

Unless `--no-validate` is supplied, the `build-config` subcommand shall invoke the SAT
engine (equivalent to `feature-check --deep`) on the named configuration before producing
any output. If the configuration is found to be invalid — e.g. a mandatory feature is
unselected, a constraint is violated, or a forced feature is deselected — the subcommand
shall write a human-readable diagnostic to standard error and exit with a non-zero status
without producing any build output.

## Rationale

Generating build flags from an incoherent configuration silently propagates a modelling
error into the compiled artefact. Aborting early prevents a broken configuration from
reaching a build system that has no visibility into the model's constraints.
