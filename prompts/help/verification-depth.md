# verification-depth — verification independence report

## SYNOPSIS
    syscribe -m <root> verification-depth
        [--sil <v>] [--status <s>] [--min-levels N] [--config <C>]
        [--plan TP-X] [--json]

## DESCRIPTION
For each requirement, reports the distinct verification levels (testLevels of its
active verifying TestCases), a count, and a depth flag: none (no active test),
hil-only (only L5), single (one level), or ok (≥2 levels). Diversity of
verification is a core SIL-4 expectation.

## OPTIONS
    --sil <v>        Restrict to requirements at silLevel/asilLevel v.
    --status <s>     Restrict to requirements whose status: equals s.
    --min-levels N   Gate: exit non-zero if any reported requirement has < N levels.
    --config <C>     Project onto a Configuration (id/qname or 'Features::A,…') —
                     only requirements active in that variant are reported.
    --plan TP-X      Restrict to a TestPlan's in-scope requirements and member
                     TestCases; composes with --config.
    --json           Emit {id, silLevel, asilLevel, levels, count, flag} array.

## EXAMPLES
    syscribe -m model/ verification-depth --sil 4
    syscribe -m model/ verification-depth --sil 4 --min-levels 2   # CI gate
    syscribe -m model/ verification-depth --config CONF-UAV-DELIVERY-001   # variant-scoped

## EXIT CODES
    0  all reported requirements meet --min-levels (or no gate)
    2  a requirement has fewer than N distinct levels

## SEE ALSO
    who-verifies, trace, audit
