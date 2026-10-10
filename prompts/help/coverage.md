# coverage — roll test coverage up the derivation tree

## SYNOPSIS
    syscribe -m <root> coverage tree <req> [--json]

## DESCRIPTION
`coverage tree` shows how well a requirement and everything derived below it
(`derivedFrom:` children, recursively) is verified. Computed on the fly, never
stored.

A **leaf** is `verified` (an active verifying TestCase), `planned` (only draft
TestCases) or `unverified`; this is the same classifier `matrix` and the
coverage report use. A **parent** aggregates its leaf descendants
(`leaves <active>/<total> active, <n> planned`) and reports the number of
direct verifying TestCases. Leaves that apply in no configuration are left out
of the counts.

Verdict glyphs: `●` all leaves verified and the node has a direct test (a leaf:
verified); `○` nothing verified, planned or direct; `◐` anything between —
including a parent whose leaves are all verified but which has no direct
integration/acceptance test, because roll-up never replaces a requirement's own
test.

    ◐ REQ-A-001  leaves 1/3 active, 1 planned | direct tests 0 [approved]
      ● REQ-A-002  verified | direct tests 1 [approved]
      ◐ REQ-A-003  planned | direct tests 1 [approved]
      ○ REQ-A-004  unverified | direct tests 0 [approved]

## OPTIONS
    --json     the same tree as JSON (leavesActive/leavesPlanned/leavesUncovered,
               directTests, verdict, children)

Exit 0 · 1 when `<req>` does not resolve to a requirement or the subcommand is
not `tree`.

## SEE ALSO
    matrix, trace, audit
