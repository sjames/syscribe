# reviews — list formal review records (ReviewRecord) and coverage

## SYNOPSIS
    syscribe -m <root> reviews [<qname>] [--open-only] [--json]
    syscribe -m <root> reviews --coverage [--json]

## DESCRIPTION
Lists `ReviewRecord` elements (§19) — formal review events (design/requirements/
hazard/test-readiness reviews, inspections, walkthroughs) that link to the model
elements they cover. Each record is a baselined traceability anchor; the review
discussion itself lives in the tool named by `recordedAt:` (e.g. a GitHub PR).

With a positional `<qname>`, lists only the reviews covering that element.

## OPTIONS
    --open-only   Only reviews that have at least one action item with disposition: open.
    --coverage    Native-Requirement review-coverage cross-table (covered ✓ / ✗ + reviews).
    --json        Emit JSON instead of a Markdown table.

## SEE ALSO
    review, safety-case
