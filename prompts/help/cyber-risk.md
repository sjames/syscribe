# cyber-risk — ISO/SAE 21434 risk determination

## SYNOPSIS
    syscribe -m <root> cyber-risk [--config <C>] [--json]

## DESCRIPTION
Lists each ThreatScenario with its computed risk: severity (max damageSeverity
over its damageScenarios) × attackFeasibility → low/medium/high/critical, its
riskTreatment, whether it is addressed by a CybersecurityGoal, and a flag
(untreated when it would trip W031).

## OPTIONS
    --config <C>   Project onto a Configuration (id/qname or 'Features::A,…') —
                   only ThreatScenarios active in that variant are listed.
    --json         Emit {id, severity, feasibility, risk, treatment, addressed, flag} array.

## EXAMPLES
    syscribe -m model_auto/ cyber-risk
    syscribe -m model_auto/ cyber-risk --json
    syscribe -m <root> cyber-risk --config <CONF-id>   # variant-scoped (needs a product line)

## NOTES
An untreated high/critical threat raises W031, and a CybersecurityGoal with a
CAL below its threats' risk raises W032, both in `validate`.

## SEE ALSO
    co-analysis, validate (W031/W032), spec safety
