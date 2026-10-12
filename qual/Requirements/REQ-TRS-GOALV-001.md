---
id: REQ-TRS-GOALV-001
type: Requirement
name: Tool shall give each safety goal a quantitative verdict and compare the fault tree with the campaign
status: draft
reqDomain: software
verificationMethod: test
---

A `SafetyGoal` with a quantified fault tree is judged on one figure in its mode; a `SafetyGoal.target:` and `[reliability]` in `.syscribe.toml` set the limit; `failureRate` carries its provenance; and `inject --consistency` compares the tree with the executable model (spec § Safety targets).

The tool **shall**:

- judge a continuous-mode goal whose events declare `diagnosticCoverage` on the first-order PMHF/PFH exactly as before, and one without coverage data on a rate-only PFH, the exact top-event probability over `missionTime` divided by it;
- judge a goal with `target.mode: demand` on PFDavg = Σ over the minimal cut sets of Π λ·(1−DC)·TI^k/(k+1) over its `proofTestInterval`;
- take the limit from the goal's `target.value`, else `[reliability.targets]`, else the built-in table, and name the source;
- report a figure as not credible when a contributing event or the top event fails with a probability above the plausibility bound over the mission time, label a verdict resting on rates that are not `verified`/`secondary`, and warn (`W988`) on a claimed rate above the bound;
- report a malformed `target:` as `E985`, a `rateStatus` outside the vocabulary as `E986` and a malformed `[reliability]` entry as `W989`;
- with `inject --ft <FT> --consistency`, report every minimal cut set with its executable verdict and every defeating set that contains no minimal cut set as missing from the tree, within `--consistency-order` and `--max-runs`.

**Source:** GH #343, from the grid-tie inverter model (`model_gti/`): `metrics` stayed silent on SG-GTI-001 because no event declares coverage, the 1.4e-1 figure over 15 years was never questioned, and nothing compared the fault tree with the campaign.

**Acceptance criteria:**

- Two events of 1e-7 and 2e-7 /h under OR over 10 000 h give a PFH of 2.996e-7 /h and pass SIL 2; with `mode: demand` and an 8760 h interval, a PFDavg of 1.314e-3.
- An event of 1e-4 /h claimed `verified` is `W988` and makes the figure not credible.
- A tree without the set the model defeats reports it as missing from the tree, and a cut set the model survives as conservative or a model gap.
