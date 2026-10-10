---
type: Requirement
id: REQ-TRS-RUNHIST-001
name: "ingest-results can retain named runs and results diff compares them"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - evidence
---

`ingest-results --run <id>` shall retain the ingested verdicts under a run identity, and `results runs` / `results diff` shall make regressions between runs visible (GH #258, v1: run identity and history; per-configuration verdicts, the `--results-as-of` lens and retained messages follow).

## Behavior

- Without `--run`, ingestion behaves exactly as before and writes no history.
- With `--run <id>`, after the normal merge, the ingest's section (`by_leaf` for function-level formats without class-qualified duplicates, `by_scenario` for session-log) is stored in `.syscribe/results-history.json` under that run id with its timestamp, format and source. Re-ingesting the same run id and kind replaces that section; the other section and other runs are kept. The sidecar `results.json` is unchanged in shape.
- `results runs [--json]` lists the retained runs (id, time, sections, counts), oldest first.
- `results diff <A> <B> [--json] [--fail-on-regression]` compares two runs per test: **regressions** (failing in B, not failing in A, including tests new in B), **fixed** (failing in A, passing in B), **still failing**, and **other changes** (any other difference, including tests dropped from B). `--fail-on-regression` exits 1 when there is a regression. An unknown run id exits 1.
