---
type: Requirement
id: REQ-TRS-FAULT-002
name: "A fault-injection campaign runs the safety invariants of a composite under every single fault and every minimal cut set"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - safety
  - fault-injection
---

`inject <composite|TestCase>` shall run the safety invariants of a composite under sets of injected faults and report which sets the design survives (GH #306, phase 2). Design: `docs/design/fault-injection.md`.

## Behavior

- **Invariants.** A scenario of the composite's TestCase tagged `@invariant` (a Gherkin tag on the line before `Scenario:`, or before `Feature:` for all of them) is a safety property that must hold whatever is injected. The Gherkin extractor records tags; a tag is case-insensitive.
- **Baseline.** The invariants must pass on the composite without faults, else the campaign stops with an error (exit 2) naming the scenario: a property that is false without faults says nothing about the faults.
- **Fault sets.** Every fault (FMEA row or fault-tree event) that applies to the composite is a single-fault set. With `--ft <FT-id>` and `--max-order N` (default 2) every minimal cut set of that fault tree up to order N whose events all have a fault model is a set of its events' faults; a cut set with an event that has no fault model is reported as not injectable, not run. `--fmea <FMEA-id>` restricts the single faults to the rows of that sheet. Identical sets are run once.
- **Run.** For each set the faults are injected at time 0, before the first step, and every invariant scenario runs unchanged. A set is **survived** when every invariant passes, **defeated** when one fails (the scenarios and messages are reported), and **error** when the run could not be made (a fault that does not fit, an invariant that errors).
- **Coverage of the analysis.** The report lists the FMEA rows of the composite's parts that have a fault model and were exercised, and those that have none (prose only) and so cannot be exercised.
- **Output and exit.** Text and `--json` (`{composite, hash, invariants, sets: [...], notInjectable, fmea}`). Exit 0 when every set is survived, 2 when any is defeated or errored (`--report-only` always exits 0 for those), 1 for usage errors (no composite, no `@invariant` scenario, an unknown fault tree or sheet).
- **Determinism.** Fixed order (single faults in element order, then cut sets by order); the same model gives the same report.
- **Not in v1:** faults injected later than time 0, randomised or sampled fault sets, quantitative use of probabilities.
