---
id: REQ-TRS-XREF-001
type: Requirement
name: Tool shall resolve absolute qualified names from the model root
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** resolve cross-references that begin with a top-level package name by splitting on `::` and traversing the element graph from the model root.

**Source:** §11.5 ¶1

**Acceptance criteria:** A reference `VehicleSystem::Powertrain::Engine` in any element resolves to the element at `model/VehicleSystem/Powertrain/Engine.md`.
