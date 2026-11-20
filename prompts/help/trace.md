# trace — full traceability slice for a requirement

## SYNOPSIS
    syscribe -m <root> trace <qname|req-id> [--linked-only] [--config <C>]

## DESCRIPTION
Shows a requirement's complete traceability slice: parents (derivedFrom), the
breakdown ADR, the SafetyGoal it derives from, the architecture that satisfies
it, and the TestCases that verify it. When a results sidecar is present, each
verifying TestCase is annotated with its ingested verdict ([pass]/[fail]/[unknown]).
Evidence status: every linked TestCase is listed whatever its status; the coverage
commands differ: `audit` and `matrix` count non-draft TestCases, while `verification-depth` and
`behavioral-coverage` count only `active` ones, so a requirement shown here can still read as
unverified there.

A "Custom links" section lists the user-defined links (`links:`, declared in
`[linkTypes]` of `.syscribe.toml`) leaving and entering the requirement. Instances of
a type that `extends` a built-in link (e.g. `satisfies`) also feed the matching
reverse index shown above unless the type declares `coverage = false`.

## REQUIRED COVERAGE
When results are loaded and a non-retired TestPlan that declares `conditions:`/`points:`/`minUnits:` includes one
of the requirement's verifying TestCases, `trace` adds `Required coverage: **pass|incomplete|fail** (worst over N
plans)` under "Verified by", then one line per plan that is not passing with its first gaps
(`TP-X incomplete: TC-Y [ambient=hot] (missing)`). Only the plan's gaps for this requirement's own verifiers
count. A requirement no such plan covers shows nothing more. See `syscribe help testplan`.

## OPTIONS
    --linked-only   Ignore ingested results; show linked tests without verdicts.
    --config <C>    Configuration lens (REQ-TRS-PROJ-001): trace over only the
                    elements active in configuration C (a stored Configuration
                    id/qname or an ad-hoc 'Features::A,Features::B' set).
                    Inactive satisfiers/verifiers/parents are omitted; if the
                    requirement itself is inactive in C the command exits 1.

## EXAMPLES
    syscribe -m model/ trace REQ-UAV-NAV-001
    syscribe -m model/ trace REQ-UAV-NAV-001 --linked-only
    syscribe -m model/ trace REQ-UAV-NAV-001 --config CONF-UAV-SURVEY-001

## SEE ALSO
    why, who-verifies, links, verification-depth, follow, link-types
