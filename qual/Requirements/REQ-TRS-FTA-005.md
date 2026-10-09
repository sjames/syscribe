---
id: REQ-TRS-FTA-005
type: Requirement
name: Tool shall drive hardware safety metrics from fault-tree cut sets and gate the verdict in the exit code
status: draft
reqDomain: software
verificationMethod: test
---

`metrics` **shall** evaluate each `SafetyGoal`'s fault tree through its minimal cut sets: only reachable, non-`house` events in a cut set with a non-negative `failureRate` contribute; an event in an order-1 cut set is single-point (residual rate `lambda*(1-DC)`), an event only in higher-order cut sets contributes no residual rate; `PMHF = lambda_RF + lambda_DPF` where `lambda_DPF` is the dual-point rate over the tree's `missionTime`, not the latent rate. A contributing event without `diagnosticCoverage` (or, once any event declares it, without `latentDiagnosticCoverage`) is treated as 0 and raises `W965`; a dual-point term without a `missionTime` raises `W966`. SIL 1 is gated (PFH < 1e-5). A goal with computed metrics but no recognised target shows verdict `no target`. `metrics` **shall** exit 1 when any goal's verdict is `fail`.

**Source:** GitHub issues #211 and #213.

**Acceptance criteria:** `AND(a,b)` and `OR(a,b)` over the same events yield different SPFM/PMHF; `metrics` exits non-zero on a failing goal and zero on a passing one; `W965` is raised for the event lacking DCl.
