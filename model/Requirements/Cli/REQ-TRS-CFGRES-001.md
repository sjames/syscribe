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
- Every evidence reader that judges one configuration uses the effective results: each `matrix` configuration column (a requirement whose test fails on `CONF-A` and passes on `CONF-B` is `failing` in the first column and `passing` in the second), and, under `--config CONF-X`, `trace`, `safety-case`, `audit`, `coverage tree` and `matrix --rollup`. Without a configuration, readers are conservative: a function's or scenario's verdict is the worst over the global sections and every configuration (Fail > Flaky > Pass > Ignored), so a failure on any variant is never hidden; rollups (`coverage tree`, `matrix --rollup`, `audit`) judge each configuration on its own effective results and AND them like the matrix. `validate --all-configs` and `audit --all-configs` judge each variant on its own.
- With `--run`, the run record keeps the per-configuration sections as well; `--results-as-of` restores them, and `results diff` compares the global sections and, for every configuration either run holds evidence for, the two runs' effective results (a run without that configuration's section contributes its global results), naming a changed test `<test> @ <config>`. `results runs` lists each run's per-configuration counts and `results failures` lists the failing functions of every configuration (`<function> @ <config>`), or those of one configuration under `--config`.
- A sidecar or history written before this feature reads unchanged.
