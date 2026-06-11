---
id: REQ-TRS-TRACE-006
type: Requirement
title: "Tool shall emit E313 for incompatible domain/reqDomain in satisfies: links"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit error `E313` when an architecture element with `domain: X` carries `satisfies:` pointing to a `Requirement` with `reqDomain: Y`, where X and Y are incompatible. Compatibility rules: `system` domain is compatible with any `reqDomain`; `hardware` is only compatible with `hardware` or `system`; `software` is only compatible with `software` or `system`.

**Source:** §12.5; §11.12 `E313`

**Acceptance criteria:** A `software` element satisfying a `hardware` requirement produces `E313`. A `software` element satisfying a `system` requirement does not.
