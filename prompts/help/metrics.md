# metrics — quantitative safety targets per goal (SPFM/LFM/PMHF, PFH, PFDavg)

## SYNOPSIS
    syscribe -m <root> metrics [--config <C>] [--sensitivity] [--json]

## DESCRIPTION
Judges every SafetyGoal on one figure from the FaultTree whose topEvent is that goal:

  - continuous mode, a contributing event declares diagnosticCoverage: the ISO 26262-5
    first-order roll-up (SPFM, LFM when latentDiagnosticCoverage is given, PMHF/PFH =
    lambda_RF + lambda_DPF), unchanged;
  - continuous mode, no coverage data: a rate-only PFH, the exact (BDD) top-event
    probability over the tree's missionTime divided by it;
  - `target: {mode: demand, proofTestInterval: ...}` on the goal: PFDavg = sum over the
    minimal cut sets of prod lambda*(1-DC)*TI^k/(k+1).

The roll-up is driven by the tree's minimal cut sets (see `fault-tree analyze`): gate
logic matters, only reachable non-house events with a non-negative failureRate in a cut
set contribute. Missing diagnosticCoverage / latentDiagnosticCoverage count as 0 (W985).

The limit is the goal's own `target.value`, else `[reliability.targets.asil.<A-D>]` /
`[reliability.targets.sil.<1-4>]` in .syscribe.toml (spfm, lfm, pmhf or pfh, pfd), else
the built-in ISO 26262 / IEC 61508 table, and the report names which. SIL 1..4: PFH
1e-5..1e-8 /h, PFDavg 1e-1..1e-4.

Verdicts: pass, fail, `not credible`, `no target`, n/a. In continuous mode a figure is not
credible when a contributing event, or the top event, fails with a probability above
`[reliability] plausibleProbability` (default 0.1) over the mission time: the figure is no
longer a rate. In demand mode the test is lambda*(1-DC)*TI per event, and the PFDavg itself.
A demand-mode goal whose level has no PFDavg limit (the ASIL table has none) is `no target`
until `target.value` or `[reliability.targets.<scheme>.<level>] pfd` gives one. A verdict
that rests on rates whose `rateStatus` is not verified or secondary says "(unverified
rates)". The report also gives the exact top-event probability, the largest contributor
(Fussell-Vesely importance: the share of the top event involving the event, the rate to
check first) and, when different, the event with the highest Birnbaum importance. A
malformed `target:` (E985) is ignored and the report says so.

## OPTIONS
    --config <C>     Project onto a Configuration (id/qname or 'Features::A,…') —
                     metrics are computed only over goals active in that variant.
    --sensitivity    Per goal, every contributing event's figure and verdict with its
                     lambda divided and multiplied by 10; marks the events that move the verdict.
    --json           Emit an array of {id, asil, sil, spfm, lfm, pmhf, lambdaDpf, pass, verdict,
                     mode, figure{metric,value,basis}, target{value,source}, rateOnly,
                     topEventProbability, missionHours, unverifiedRates, notCredible,
                     gates, targetIgnored, largestContributor, highestBirnbaum[, sensitivity]}.

## EXAMPLES
    syscribe -m model_auto/ metrics
    syscribe -m model_gti/ metrics --sensitivity
    syscribe -m model_sil/ metrics --json

## NOTES
Inputs: FaultTreeEvent.failureRate (lambda/h), diagnosticCoverage, latentDiagnostic-
Coverage, rateSource/rateStatus, SafetyGoal.target. Exit status 2 when a goal fails on a
gating figure: the first-order roll-up always gates; a rate-only or PFDavg figure gates
only when the goal declares `target:` or .syscribe.toml sets the limit, otherwise its
`fail` is advisory (exit 0, noted on stderr). The same rule decides W033 in `validate`.
Usage errors exit 1.

## SEE ALSO
    validate (W033, W985, W986, E985, E986, W988, W989), fault-tree analyze, inject
    --consistency, audit, safety-case, spec safety
