---
type: Requirement
id: REQ-TRS-RESASOF-001
name: "--results-as-of reads a retained run's verdicts instead of the latest sidecar"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - evidence
---

A global `--results-as-of <run>` option shall make the evidence-reading commands use the verdicts of a retained run (GH #258, lens).

## Behavior

- `--results-as-of <run>` (or `--results-as-of=<run>`) may appear anywhere on the command line, like `-m`. The run must have been retained with `ingest-results --run`; an unknown run exits 1 with the list of retained runs, and a run without history exits 1 as well.
- `matrix`, `trace`, `testplan`, `safety-case`, `audit`, `validate` and every other command that reads ingested results then read that run's `by_leaf` / `by_scenario` verdicts instead of `.syscribe/results.json` (class-qualified keys are not retained, so a qualified reference resolves through its leaf name). The sidecar and the history are not modified.
- Commands that do not read results are unaffected.
