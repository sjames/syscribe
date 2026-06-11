---
id: REQ-TRS-TRACE-008
type: Requirement
title: Tool shall emit E314 for a deployment package with no hardware Allocation
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit error `E314` when a `Part` or `PartDef` with `isDeploymentPackage: true` has no `Allocation` element in the model whose `from:` resolves to that element and whose target has `domain: hardware`.

**Source:** §12.6; §11.12 `E314`

**Acceptance criteria:** A `PartDef` with `isDeploymentPackage: true` and no associated `Allocation` to a hardware element produces `E314`. Adding a valid `Allocation` removes the error.
