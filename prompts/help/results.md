# results — retained test runs and run-to-run diffs

## SYNOPSIS
    syscribe -m <root> results runs [--json]
    syscribe -m <root> results failures [--json] [--config <CONF-id>]
    syscribe -m <root> results diff <runA> <runB> [--json] [--fail-on-regression]

## DESCRIPTION
Retain a run with `syscribe -m <root> ingest-results --run <run-id> <file>`.

`ingest-results` keeps only the latest verdict per function and scenario. With
`--run <id>` it additionally retains that ingest under a run identity in
`.syscribe/results-history.json` (the sidecar `results.json` is unchanged).
Re-ingesting the same run id replaces that run's section of the same kind (the
function-level and the session-log sections are independent); other runs are kept.
Without `--run` nothing is retained.

`results failures` lists the failing, skipped and flaky functions of the latest
ingest with the retained JUnit `message` and `time` (passing cases keep no detail). With
per-configuration evidence (`ingest-results --config`) it lists every configuration's
failures as `<function> @ <config>`, or one configuration's effective results under `--config`.

`results runs` lists the retained runs, oldest first. `results diff A B` compares
two runs per test:

- **Regressions** — failing in B and not failing in A (including tests new in B)
- **Fixed** — failing in A, passing in B
- **Still failing** — failing in both
- **Other changes** — any other difference, including tests that dropped out of B

`--fail-on-regression` exits 1 when there is a regression or a failing test vanished
or was skipped in B (CI gate). Only the sections (function-level, session-log) that
both runs hold are compared. Run ids must be non-empty and not start with `-`. An unknown
run id exits 1.

**`--results-as-of <run>`** is a global option (like `-m`, anywhere on the command line): the
evidence-reading commands (`matrix`, `trace`, `testplan`, `safety-case`, `audit`, `validate`, …)
then use that retained run's verdicts instead of the latest sidecar, e.g.
`syscribe -m model validate --results-as-of SW-0.9.0-rc3`. An unknown run exits 1 and lists
the retained ones. Class-qualified JUnit keys are not retained, so a qualified reference resolves
through its leaf name, and a retained run keeps no failure messages. Nothing on disk changes.
Not applied: `mcp`, `lsp` and `ingest-results` refuse the option; the derived safety/traceability
diagrams and the commands that write the model (`set`, `aw`, `baseline` seal) still read the latest
sidecar; an explicit `validate --results <file>` wins over the lens.

`trace` and `safety-case` show the retained message and time of a failing verifier ("Failing tests" / "Failure details").

Not yet: per-configuration verdicts (GH #258).

## SEE ALSO
    ingest-results, audit, matrix
