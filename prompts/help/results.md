# results — retained test runs and run-to-run diffs

## SYNOPSIS
    syscribe -m <root> ingest-results [--format F] --run <run-id> <file>
    syscribe -m <root> results runs [--json]
    syscribe -m <root> results diff <runA> <runB> [--json] [--fail-on-regression]

## DESCRIPTION
`ingest-results` keeps only the latest verdict per function and scenario. With
`--run <id>` it additionally retains that ingest under a run identity in
`.syscribe/results-history.json` (the sidecar `results.json` is unchanged).
Re-ingesting the same run id replaces that run's section of the same kind (the
function-level and the session-log sections are independent); other runs are kept.
Without `--run` nothing is retained.

`results runs` lists the retained runs, oldest first. `results diff A B` compares
two runs per test:

- **Regressions** — failing in B and not failing in A (including tests new in B)
- **Fixed** — failing in A, passing in B
- **Still failing** — failing in both
- **Other changes** — any other difference, including tests that dropped out of B

`--fail-on-regression` exits 1 when there is a regression (CI gate). An unknown
run id exits 1.

Not yet: per-configuration verdicts, a `--results-as-of <run>` lens on
`matrix`/`trace`/`audit`, and retained failure messages (GH #258).

## SEE ALSO
    ingest-results, audit, matrix
