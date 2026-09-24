# trace — full traceability slice for a requirement

## SYNOPSIS
    syscribe -m <root> trace <qname|req-id> [--linked-only]

## DESCRIPTION
Shows a requirement's complete traceability slice: parents (derivedFrom), the
breakdown ADR, the SafetyGoal it derives from, the architecture that satisfies
it, and the TestCases that verify it. When a results sidecar is present, each
verifying TestCase is annotated with its ingested verdict ([pass]/[fail]/[unknown]).

A "Custom links" section lists the user-defined links (`links:`, declared in
`[linkTypes]` of `.syscribe.toml`) leaving and entering the requirement. Instances of
a type that `extends` a built-in link (e.g. `satisfies`) also feed the matching
reverse index shown above unless the type declares `coverage = false`.

## OPTIONS
    --linked-only   Ignore ingested results; show linked tests without verdicts.

## EXAMPLES
    syscribe -m model/ trace REQ-UAV-NAV-001
    syscribe -m model/ trace REQ-UAV-NAV-001 --linked-only

## SEE ALSO
    why, who-verifies, links, verification-depth, follow, link-types
