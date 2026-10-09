# cyber-risk — ISO/SAE 21434 risk determination

## SYNOPSIS
    syscribe -m <root> cyber-risk [--config <C>] [--json | --format md|html|json]

## DESCRIPTION
Lists each ThreatScenario with its computed risk: severity (max damageSeverity
over its damageScenarios) × attackFeasibility → low/medium/high/critical, its
riskTreatment, whether it is addressed by a CybersecurityGoal, and a flag
(untreated when it would trip W031).

## OPTIONS
    --config <C>   Project onto a Configuration (id/qname or 'Features::A,…') —
                   only ThreatScenarios active in that variant are listed.
    --json         Emit {id, severity, feasibility, risk, treatment, addressed, flag} array.
    --format md|html|json
                   Render the impact x feasibility risk matrix (heat table) with the
                   threats placed in their cells, using the configured [cyber] method;
                   html is a standalone page. The default output is unchanged.

## EXAMPLES
    syscribe -m model_auto/ cyber-risk
    syscribe -m model_auto/ cyber-risk --json
    syscribe -m model_auto/ cyber-risk --format html
    syscribe -m <root> cyber-risk --config <CONF-id>   # variant-scoped (needs a product line)

## NOTES
Configurable via [cyber] in .syscribe.toml (method simple|annex, risk_matrix,
cal_table, cal_by_risk, attack_potential); the annex tables are EXAMPLE tables,
not normative - verify against your copy of ISO/SAE 21434. With a [cyber] table the
output adds Vector / Risk value / Expected CAL (JSON: method, attackVector,
riskValue, expectedCal). Malformed entries -> W640, defaults apply.

An untreated high/critical threat raises W031, and a CybersecurityGoal with a
CAL below its threats' risk raises W032, both in `validate`.

## SEE ALSO
    hara, co-analysis, validate (W031/W032), spec safety
