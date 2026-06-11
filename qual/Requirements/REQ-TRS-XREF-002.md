---
id: REQ-TRS-XREF-002
type: Requirement
title: Tool shall resolve relative references outward from the current package
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** resolve a cross-reference that does not begin with a known top-level package name by searching: (a) the current element's package, (b) each enclosing package outward, (c) any imported packages. A `./` prefix restricts resolution to siblings of the current element.

**Source:** §11.5 ¶2–3

**Acceptance criteria:** A sibling reference `Engine` within package `Powertrain` resolves to `Powertrain::Engine`; the same reference from an outer package does not resolve unless `Powertrain` is imported.
