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
- With `--run <id>`, after the normal merge, the ingest's section (`by_leaf` for function-level formats without class-qualified duplicates, `by_scenario` for session-log) is stored in `.syscribe/results-history.json` under that run id with its timestamp and source. Re-ingesting the same run id and kind replaces that section; the other section and other runs are kept. The sidecar `results.json` is unchanged in shape.
- A records-like ingest (`records`, `robot`, `cucumber`, a results plugin) also keeps its records, with their measurements, in the run's own file `.syscribe/runs/<run>.json` (GH #471), globally or per Configuration with `--config`; the run id is written as a file name with every byte but letters, digits, `_`, `-` and `.` (not first) as `%XX`. `results-history.json` is the index: per run the verdict maps, sources, time, a record count and a `records_dropped` flag. Function-level and session-log ingests write no run file. `trade-study --run <id>` reads them (`REQ-TRS-MOEMEAS-001`).
- A retained record is the slim record: the key (test case, scenario, environment), verdict, rationale, measurements, the hashes it was stamped with and the environment and start and finish times; not the steps, artefact references, operator, tool or build, which the sidecar keeps. A later ingest of the same run supersedes the records with the same key and keeps the others, all of them under `--replace`, and rewrites only its own run's file.
- The history is bounded: `[results] keep_run_records` (default 20, `0` keeps none; a malformed value is a warning and the default applies) is how many of the most recently ingested runs keep their records. A run the ingest touched is always the newest. An older run's file is deleted, the index keeps its verdicts with `records_dropped`, and ingest says which runs were dropped; `results runs` shows each run's record count or `records dropped`, and `results diff` still compares it.
- What an ingest writes is decided before anything is written: the run files, the index, and the files to delete are all checked against the write layer's guard first, so a protected path refuses before the sidecar is written. Files are written atomically, run files before the index and deletions after it, so a crash leaves orphan files the index ignores, never an index that points at nothing. An unreadable run file is an error and is never overwritten.
- `results runs [--json]` lists the retained runs (id, time, sections, counts), oldest first.
- `results diff <A> <B> [--json] [--fail-on-regression]` compares two runs per test: **regressions** (failing in B, not failing in A, including tests new in B), **fixed** (failing in A, passing in B), **still failing**, and **other changes** (any other difference, including tests dropped from B). Only the sections (function-level, session-log) both runs hold are compared. `--fail-on-regression` exits 1 when there is a regression or a failing test vanished or was skipped in B. `--run` needs a non-empty id not starting with `-`. An unknown run id exits 1.
