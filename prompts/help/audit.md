# audit — safety-readiness dashboard

## SYNOPSIS
    syscribe -m <root> audit [--json] [--profile <name>]
        [--config <C> | --all-configs] [--plan TP-X]

## DESCRIPTION
Rolls up a top-level readiness picture (coverage and orphans count non-draft TestCases;
`verification-depth` and `behavioral-coverage` count `active` ones only; `trace` and
`who-verifies` list linked TestCases of any status): requirement status split (overall and
per top-level package), SIL/ASIL distribution, per-configuration coverage %, coverage by requirement class (the derivation-tree
roll-up of `matrix --rollup`: complete / partial / none per `reqClass`, honouring the
`[coverage]` policy; `coverageByClass` in JSON),
orphans (requirements with no test / no satisfying element, dangling TestCases,
no-trace requirements), Safety (hazards, goals by integrity level, FTA, FMEA,
hardware metrics) and Security (TARA, CAL, vulnerabilities, zones) sections,
and a single PASS/FAIL verdict. When test results are ingested it adds a
Verification results section (active tests by verdict, goals supported /
incomplete / failing, plans by verdict; `verification` in JSON, null without results).

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
mechanism) is present, W312 (an approved/implemented requirement whose active
verifier fails), a safety goal is FAILING on ingested results, W033 (hardware metric below target) or W805 (no derived
requirement) is present on an ASIL C/D goal, or — with --profile — any finding
the profile promotes. Configure with an [audit] table in .syscribe.toml:
    [audit]
    fail_on = ["W306", "W312"]  # codes failing at any level (the defaults)
    [audit.fail_on_asil]        # code = ASIL levels; empty table opts out
    W033 = ["C", "D"]
    W805 = ["C", "D"]

## EXAMPLES
    syscribe -m model/ audit
    syscribe -m model/ audit --config CONF-UAV-DELIVERY-001   # variant-scoped readiness
    syscribe -m model/ audit --all-configs                    # gate every variant
    syscribe -m model/ audit --json
    syscribe -m model_mg/ audit --profile magicgrid           # model_mg/ defines [profiles.magicgrid]

## EXIT CODES
    0  PASS    2  FAIL (verdict, or any variant under --all-configs)    1  undefined --profile / bad --config

## SEE ALSO
    validate, matrix, verification-depth, metrics
