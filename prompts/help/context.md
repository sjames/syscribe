# context — one bounded briefing bundle for an element

## SYNOPSIS
    syscribe -m <root> context <ref> [--depth N] [--max-bytes N] [--format text|markdown|json]

## DESCRIPTION
Everything needed to start work on an element in one call, instead of `show`, `trace`, `links`,
`who-verifies` and `validate` and a decision about what to fetch next. It prints:

- the element: id, type, status, one-line summary, file and body;
- upstream (what it links to) and downstream (what links to it) neighbours to `--depth`, each with
  id, type, status, the relationship and a one-line summary;
- verifying tests with the latest verdict and the environment, build and time it ran on;
- for a `PlanningItem`: parent and children, `achieves`, evidence (a `commit:` entry with its
  resolved full sha and subject line, GH #455), blockers, claim, assignee and
  open questions;
- the validation findings that sit on the element and on the neighbours listed, errors first;
- the qualified-name parents.

The output never exceeds `--max-bytes`. When it would, the least important items go first —
downstream, upstream, parents, findings, verification, planning, and last the body — and a final
notice (`truncated` in JSON) names what was dropped and how many.

## OPTIONS
    --depth <N>       How far to follow links in each direction (default 1; 0 lists none).
    --max-bytes <N>   Upper bound on the output (default 16000, minimum 1024).
    --format <f>      text or markdown (the same), or json (schemaVersion 1.0). --json is the same as --format json.

An unknown reference exits 1; a bad option or value exits 2.

## EXAMPLES
    # against the bundled PlanningItem example (examples/planning-item/model/)
    syscribe -m examples/planning-item/model/ context PI-RTH-DOCS-001
    syscribe -m examples/planning-item/model/ context PI-RTH-DOCS-001 --depth 2 --max-bytes 4000 --format json

## SEE ALSO
    show, trace, links, who-verifies, steps, validate
