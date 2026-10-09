# safety-case — GSN goal → argument → evidence tree

## SYNOPSIS
    syscribe -m <root> safety-case [<SG-id>] [--config <C>] [--no-implicit] [--json]

## DESCRIPTION
Renders the assurance argument for each SafetyGoal (or the one given): the
Argument nodes (claim/strategy/solution/context/justification/assumption/undeveloped) that support it and their evidence
(Requirements, TestCases with ingested verdicts, sub-Arguments, AssumptionOfUse).
It always folds in the implicit SafetyGoal ← Requirement (derivedFromSafetyGoal) ←
derivedChildren* ← TestCase (verifies) chain (even when a goal has Arguments; a
requirement an Argument already cites is shown once). Nodes with no evidence are
marked [UNDEVELOPED]; each goal gets a SUPPORTED/INCOMPLETE/FAILING verdict and a
Completeness summary follows. An unknown <SG-id> exits 1.

## OPTIONS
    --config <C>   Project onto a Configuration (id/qname or 'Features::A,…') —
                   only goals and evidence active in that variant are assembled.
    --no-implicit  Drop the implicit SafetyGoal ← Requirement ← TestCase fold-in;
                   show only the explicit Argument/AssumptionOfUse structure.
    --json         Emit {goals:[{id,title,verdict,status,completeness,arguments,
                   requirements,assumptions}], completeness, verdictsUnknown}.

## EXAMPLES
    # against the bundled automotive model (model_auto/)
    syscribe -m model_auto/ safety-case
    syscribe -m model_auto/ safety-case SG-ENG-001 --json
    syscribe -m model_auto/ safety-case SG-ENG-001 --no-implicit
    # variant-scoped (model_auto/ has no product line; any model with Configurations)
    syscribe -m <root> safety-case --config <CONF-id>

## SEE ALSO
    trace, co-analysis, spec safety
