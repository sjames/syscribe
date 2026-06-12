---
id: REQ-TRS-XREF-004
type: Requirement
name: Tool shall detect and report circular supertype chains without crashing
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** detect cycles in `supertype:` and `typedBy:` chains (e.g. A specializes B which specializes A) and emit an error finding identifying the cycle. Detection of a cycle **shall not** cause an infinite loop, stack overflow, or crash.

**Source:** §11.6

**Acceptance criteria:** A model with a two-element or three-element `supertype:` cycle produces an error and the tool exits normally.
