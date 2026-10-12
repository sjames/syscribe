---
id: REQ-TRS-VCOMP-001
type: Requirement
name: Tool shall treat a verification composite's environment parts apart from the product and accept architecture elements used directly
status: draft
reqDomain: software
verificationMethod: test
---

The subject of a scenario TestCase may be a verification composite: an ordinary `PartDef` that holds the system under test (architecture elements that carry `exhibitsStates:` themselves, and executable parts that `models:` one) and the parts of the test set-up, marked `role: environment` on their usage (spec §12.11).

The tool **shall**:

- treat a part usage typed by an architecture element as its own counterpart when it checks connection correspondence (`W991`) and as exercising the requirements that element satisfies (`W992`);
- skip an environment part when it checks connection correspondence (`W991`), and never count it as evidence for a requirement (`W992`);
- report a `models:` or `portMap:` on an environment part's usage as `W993` and ignore it;
- report a `role:` that is not a known role, or that is not on a part usage, as `E992`, whether or not the model uses `models:`;
- list environment parts apart in the `simulate --coverage` report and leave them out of its aggregate, and list them in the `inject` report, marking a fault set whose faults all lie in environment parts.

**Source:** GH #332, from the grid-tie inverter model (`model_gti/`): the protection chain mixed the grid source with the product, and the supervisor existed twice.

**Acceptance criteria:**

- A verification composite with an environment stimulus, an executable model and an architecture element used directly validates with none of `E992`, `W991`–`W993`, and its coverage and fault-campaign reports list the stimulus as an environment part.
- Each defect class gives its code.
