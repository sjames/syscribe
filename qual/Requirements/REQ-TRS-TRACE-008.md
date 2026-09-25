---
id: REQ-TRS-TRACE-008
type: Requirement
name: Tool shall emit E314 for a deployment package with no allocation to a hardware element, in any allocation form
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit error `E314` when a `Part` or `PartDef` with `isDeploymentPackage: true` is the source of no allocation edge whose target has `domain: hardware`.

The allocation edges **shall** be the §12.9 unified edge set ([[REQ-TRS-ALLOC-001]]) — every authoring form counts:

- `allocatedTo:` on the deployment package itself (form 1);
- a standalone `type: Allocation` element, top-level `allocatedFrom`/`allocatedTo` or per `features:` entry (form 2);
- a legacy `allocatedFrom:` authored on the hardware target (accepted for backward compatibility, §12.9).

Endpoints resolve by qualified name or stable id, like every other allocation consumer. Before GH #131 only a
top-level `Allocation` element (matched by exact qualified name) cleared `E314`, so the `allocatedTo:` form the
authoring prompt recommends still raised it.

**Source:** §12.6, §12.9; §11.12 `E314`; GH #131

**Acceptance criteria:**

- A `PartDef` with `isDeploymentPackage: true` and no allocation to a hardware element produces `E314`.
- Adding a valid top-level `Allocation` element removes the error.
- So does `allocatedTo:` on the package, a `features:`-form `Allocation` element, or a legacy `allocatedFrom:` on the hardware target.
- An allocation whose only target is a `software` element still produces `E314`.
