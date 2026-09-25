# metrics — quantitative HW safety metrics (SPFM/LFM/PMHF)

## SYNOPSIS
    syscribe -m <root> metrics [--config <C>] [--json]

## DESCRIPTION
Computes ISO 26262-5 hardware architectural metrics per SafetyGoal over the
FaultTreeEvents of the FaultTree whose topEvent is that goal: SPFM, LFM (when
latentDiagnosticCoverage is given), and PMHF, compared against the goal's
ASIL/SIL target. Opt-in: a goal with no diagnosticCoverage data shows n/a.
First-order FMEDA approximation — verify independently before use in a safety case.

## OPTIONS
    --config <C>   Project onto a Configuration (id/qname or 'Features::A,…') —
                   metrics are computed only over goals active in that variant.
    --json         Emit {id, asil, sil, spfm, lfm, pmhf, pass} array.

## EXAMPLES
    syscribe -m model_auto/ metrics
    syscribe -m model_sil/ metrics --json
    syscribe -m <root> metrics --config <CONF-id>   # variant-scoped (needs a product line)

## NOTES
Inputs: FaultTreeEvent.failureRate (λ/h), diagnosticCoverage, latentDiagnostic-
Coverage. A goal below target also raises W033 in `validate`.

## SEE ALSO
    validate (W033), audit, spec safety
