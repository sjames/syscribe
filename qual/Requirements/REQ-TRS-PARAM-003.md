---
id: REQ-TRS-PARAM-003
type: Requirement
title: Tool shall accept inclusive range syntax so parameter range checks are enforced
status: draft
reqDomain: software
verificationMethod: test
---

A typed feature `parameter:` may declare a numeric `range:` as a string, against which a `Configuration`'s bound value is checked (`E205`, see [[REQ-TRS-PARAM-001]]) and to which two-level `bindTo:` propagation is constrained (`E202`). The range string **shall** be parsed for both of these accepted spellings, with identical inclusive `[min, max]` semantics:

- `"min..max"` — e.g. `"1..8"`, `"900..1200"`;
- `"min..=max"` — Rust-style inclusive form, e.g. `"1..=8"`.

A range the tool fails to parse **shall not** silently disable the range check. Today `"1..=8"` is not parsed (the `=` is left attached to the upper bound and the parse fails), so the parameter's range is silently dropped and an out-of-range binding such as `99` produces no `E205` (GH #14). After this requirement, both spellings yield the same `(min, max)` bounds and the same `E205`/`E202` enforcement.

Whitespace around the bounds and the operator (e.g. `"1 ..= 8"`) **shall** be tolerated.

**Source:** §9.7; GH #14.

**Acceptance criteria:**

1. A parameter declared `range: "1..=8"` whose `Configuration` binds `99` produces `E205`; binding `8` produces none.
2. A parameter declared `range: "1..8"` behaves identically (regression — the existing dot-dot form is unchanged).
3. The same parsing applies to `bindTo:` propagation range checks (`E202`).
