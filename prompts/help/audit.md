# audit — safety-readiness dashboard

## SYNOPSIS
    syscribe -m <root> audit [--json] [--profile <name>]
        [--config <C> | --all-configs] [--plan TP-X]

## DESCRIPTION
Rolls up a top-level readiness picture: requirement status split (overall and
per top-level package), SIL/ASIL distribution, per-configuration coverage %,
orphans (requirements with no test / no satisfying element, dangling TestCases,
no-trace requirements), and a single PASS/FAIL verdict.

## OPTIONS
    --profile <name>  Use a .syscribe.toml [profiles.<name>] policy as the bar.
    --config <C>      Project the whole dashboard onto a Configuration (id/qname
                      or 'Features::A,Features::B') — verdict, W306 and coverage
                      computed only over elements active in that variant.
    --all-configs     Audit every stored Configuration's variant; exit non-zero
                      if any fails (product-line CI gate).
    --plan TP-X       Scope the verdict to a TestPlan: validate the full model
                      (no escaping-ref artifacts) and count only findings on the
                      plan's in-scope elements; sections scoped to the plan.
    --json            Emit the whole rollup as one JSON document.

## DESCRIPTION (policy)
The verdict FAILS when any Error finding exists, any W306 (unsatisfied safety
mechanism) is present, or — with --profile — any finding the profile promotes.

## EXAMPLES
    syscribe -m model/ audit
    syscribe -m model/ audit --profile safety
    syscribe -m model/ audit --config CONF-LM3S-QEMU-001   # variant-scoped readiness
    syscribe -m model/ audit --all-configs                 # gate every variant
    syscribe -m model/ audit --json

## EXIT CODES
    0  PASS    2  FAIL (verdict, or any variant under --all-configs)    1  undefined --profile / bad --config

## SEE ALSO
    validate, matrix, verification-depth, metrics
