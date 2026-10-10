# coverage — roll test coverage up the derivation tree

## SYNOPSIS
    syscribe -m <root> coverage tree <req> [--json]

## DESCRIPTION
`coverage tree` shows how well a requirement and everything derived below it
(`derivedFrom:` children, recursively) is verified. Computed on the fly, never
stored.

A **leaf** is `verified` (an active verifying TestCase), `planned` (only draft
TestCases) or `uncovered`; this is the same classifier `matrix` and the
coverage report use. A **parent** aggregates its leaf descendants
(`leaves <active>/<total> active, <n> planned`) and reports the number of
direct *active* verifying TestCases (a draft test is intent, not evidence). Leaves
are counted once however many paths reach them. Leaves that apply in no configuration are left out
of the counts.

Verdict glyphs: `●` all leaves verified and the node has a direct test (a leaf:
verified); `○` nothing verified, planned or direct; `·` nothing applies in any
configuration; `◐` anything between —
including a parent whose leaves are all verified but which has no direct
integration/acceptance test, because roll-up never replaces a requirement's own
test.

    ◐ REQ-A-001  leaves 1/3 active, 1 planned | direct tests 0 [approved]
      ● REQ-A-002  verified | direct tests 1 [approved]
      ◐ REQ-A-003  planned | direct tests 0 [approved]
      ○ REQ-A-004  uncovered | direct tests 0 [approved]

`--config` and `--plan` lenses are not supported yet (leaves are classified over all
configurations of the model, as `matrix` does); the policy rules of GH #253 and
`matrix --rollup` build on this command later.

## POLICY (`[coverage]` in `.syscribe.toml`)
How a parent is judged complete is a policy, `both` by default:

    [coverage]
    default = "both"             # direct | rollup | both
    [[coverage.rule]]            # first matching rule wins; selectors are ANDed
    asil = ["A","B","C","D"]     # also cal = ["CAL1".."CAL4"], sil = ["QM", 1, ...]
    parent_rule = "both"
    [[coverage.rule]]
    reqClass = "stakeholder"     # also tag = ["function"] (any of)
    parent_rule = "rollup"

`direct`: a direct active test is enough. `rollup`: every leaf below verified.
`both`: a direct test and every leaf. A rule that loosens an integrity-rated
requirement (ASIL A-D, CAL1-4, SIL >= 1) is a configuration error — the command
names the requirement and exits 1 — as are invalid values and unknown selectors.
The applied rule is printed on each parent line, `(rule: <rule>)`, and carried
in `--json` as `rule`.

## OPTIONS
    --json     the same tree as JSON (leavesActive/leavesPlanned/leavesUncovered,
               directTests, verdict = complete|partial|none|na, glyph, children)

Exit 0 · 1 when `<req>` does not resolve to a requirement or the subcommand is
not `tree`.

## SEE ALSO
    matrix, trace, audit
