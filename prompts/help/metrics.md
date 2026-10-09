# metrics — quantitative HW safety metrics (SPFM/LFM/PMHF)

## SYNOPSIS
    syscribe -m <root> metrics [--config <C>] [--json]

## DESCRIPTION
Computes ISO 26262-5 hardware architectural metrics per SafetyGoal over the
FaultTreeEvents of the FaultTree whose topEvent is that goal: SPFM, LFM (when
latentDiagnosticCoverage is given), and PMHF, compared against the goal's
ASIL/SIL target. Opt-in: a goal with no diagnosticCoverage data shows n/a.
First-order FMEDA approximation — verify independently before use in a safety case.

The roll-up is driven by the tree's minimal cut sets (see `fault-tree analyze`): gate
logic matters (AND vs OR differ), only reachable non-house events with a
non-negative failureRate in a cut set contribute, an order-1 event is single-point
(residual lambda*(1-DC)), and PMHF = lambda_RF + lambda_DPF where lambda_DPF is the
dual-point rate of order-2 cut sets over the tree's missionTime. Missing
diagnosticCoverage / latentDiagnosticCoverage count as 0 and raise W965.

Verdicts: pass, fail, `no target` (computed but no recognised ASIL/SIL), n/a (not
computed). SIL 1..4 gate PFH (1e-5..1e-8 /h). Exit status 2 when any goal fails.

## OPTIONS
    --config <C>   Project onto a Configuration (id/qname or 'Features::A,…') —
                   metrics are computed only over goals active in that variant.
    --json         Emit {id, asil, sil, spfm, lfm, pmhf, lambdaDpf, pass, verdict} array.

## EXAMPLES
    syscribe -m model_auto/ metrics
    syscribe -m model_sil/ metrics --json
    syscribe -m <root> metrics --config <CONF-id>   # variant-scoped (needs a product line)

## NOTES
Inputs: FaultTreeEvent.failureRate (λ/h), diagnosticCoverage, latentDiagnostic-
Coverage. A goal below target also raises W033 in `validate`; `metrics` itself exits 2
when any goal fails (usage errors exit 1).

## SEE ALSO
    validate (W033, W965, W966), fault-tree analyze, audit, spec safety
