---
type: Requirement
id: REQ-TRS-CFGRES-001
name: "Test evidence can be recorded per Configuration and is judged per Configuration"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - evidence
---

`ingest-results` shall accept `--config <id>` so a test that fails only on one variant is not a single global `fail` (GH #258, per-configuration verdicts).

## Behavior

- `ingest-results … --config CONF-X` stores the ingest in the sidecar under `by_config["CONF-X"]` (its own `by_leaf`, `by_scenario` and `details`, merged exactly like the global sections: a function-level ingest replaces that configuration's function-level section, a session log its scenarios) and leaves the global sections untouched. `CONF-X` must resolve to a `Configuration` (id or qualified name; stored by id): otherwise the command exits 1 and writes nothing. Without `--config` behaviour is unchanged.
- A configuration's **effective** results are the global ones overlaid with its own: for each function or scenario its own verdict (and detail) wins, otherwise the global one applies. No configuration inherits another's.
- Every evidence reader that judges one configuration uses the effective results: each `matrix` configuration column (a requirement whose test fails on `CONF-A` and passes on `CONF-B` is `failing` in the first column and `passing` in the second), and, under `--config CONF-X`, `trace`, `safety-case`, `audit`, `coverage tree` and `matrix --rollup`. Commands without a configuration use the global sections only.
- With `--run`, the run record keeps the per-configuration sections as well; `--results-as-of` restores them, and `results diff` compares the global sections and each configuration present in both runs (a changed test under a configuration is named `<test> @ <config>`).
- A sidecar or history written before this feature reads unchanged.
