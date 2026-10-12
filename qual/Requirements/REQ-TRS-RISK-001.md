---
id: REQ-TRS-RISK-001
type: Requirement
name: Tool shall record fault-injection campaigns and tie every defeat to a residual-risk decision
status: draft
reqDomain: software
verificationMethod: test
---

A fault-injection campaign (`inject`) reports the fault sets a composite's `@invariant` scenarios survive and those that defeat them. A `ResidualRisk` element (`RISK-*`) records the decision on one defeated fault set (spec §12.14).

The tool **shall**:

- keep a campaign with `inject --record [--run <id>] [--config <C>]` in `.syscribe/campaigns.json`: composite hash, invariants and their hashes, every set with origin, verdict and failing invariant, the sets not run, and the engine version, the same content for the same model;
- list the recorded campaigns and compare two records with `results campaigns [diff]`, and write a campaign report with the decisions and failure rates with `inject --report <file>`;
- check a `ResidualRisk`'s fields (`E942`) and references (`E943` unresolved, `E944` wrong kind);
- report a defeat in the latest recorded campaign that no proposed, accepted or mitigated decision names (`W940`), a decision the campaign contradicts (`W941`), a decision taken on another composite hash (`W942`), an expired decision (`W943`), and a record of a composite that has since changed (`W944`);
- show the campaign as a solution node in `safety-case`, and the unexplained defeats in `audit`.

**Source:** GH #341, from the grid-tie inverter model (`model_gti/`): the decision on its defeated common-cause sets lived only in prose, so a new defeat looked the same as a known one.

**Acceptance criteria:**

- `inject --record` writes a run, `results campaigns` lists it, and a second run on the same model records the same content.
- An unexplained defeat is a warning; a `ResidualRisk` for the set silences it; changing the composite makes the decision stale.
- The safety case shows the campaign node.
