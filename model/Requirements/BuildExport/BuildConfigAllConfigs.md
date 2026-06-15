---
type: Requirement
id: REQ-TRS-BUILD-018
name: "build-config --all-configs shall generate a JSON matrix of all configurations"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BUILD-010]
breakdownAdr: Decisions::BuildExportADR
tags:
  - cli
  - build-integration
  - ci
---

The `build-config` subcommand shall accept `--all-configs` as a mutually exclusive
alternative to `--config <id>`. When used, it shall:

1. Enumerate every `Configuration` element in the model.
2. Resolve the build variable set for each configuration independently.
3. Emit a JSON array to standard output, where each entry is an object with at least:
   - `"config"` — the configuration's stable `id` (e.g. `"CONF-UAV-MAPPING-001"`)
   - `"name"` — the configuration's display `name`
   - `"vars"` — the resolved build variable map for that configuration

The output is suitable for direct consumption as a GitHub Actions `matrix` input or
equivalent CI matrix mechanism, allowing a single model to drive parallel per-variant
build jobs without maintaining the configuration list separately.

SAT validation is applied to each configuration; configurations that fail validation
are included in the output with an `"error"` key explaining the failure, and the
subcommand exits with a non-zero status if any configuration is invalid.
