# co-analysis — safety ↔ security co-engineering view

## SYNOPSIS
    syscribe -m <root> co-analysis [--config <C>] [--json]

## DESCRIPTION
Shows the cross-domain overlap (ISO 26262 ⇄ ISO/SAE 21434): for each SafetyGoal /
HazardousEvent, the cyber threats that can violate it — following
ThreatScenario → damageScenarios → DamageScenario → hazardRef → goal — plus a
"gaps" section of safety-tagged DamageScenarios with no hazardRef (W030).

## OPTIONS
    --config <C>   Project onto a Configuration (id/qname or 'Features::A,…') —
                   only goals and threats active in that variant are shown.
    --json         Emit {goals:[…], unlinkedSafetyDamage:[…]}.

## EXAMPLES
    syscribe -m model_auto/ co-analysis
    syscribe -m model_auto/ co-analysis --json
    syscribe -m <root> co-analysis --config <CONF-id>   # variant-scoped (needs a product line)

## SEE ALSO
    cyber-risk, safety-case, validate (W030), spec safety
