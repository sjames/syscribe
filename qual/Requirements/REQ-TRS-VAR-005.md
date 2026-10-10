---
id: REQ-TRS-VAR-005
type: Requirement
name: Tool shall enforce per-Configuration uncovered-requirement rule W015
status: draft
reqDomain: software
verificationMethod: test
---

When the variability dimension is active (see [[REQ-TRS-VAR-001]]), the tool **shall** emit warning `W015`, **once per requirement** R (GH #245), listing every `Configuration` C in which R is *active* (R's `appliesWhen:` is satisfied by C's selections) and when **no** `TestCase` T exists with T's `appliesWhen:` satisfied by C and `R ∈ T.verifies`. The finding message **shall** identify both R and C, e.g.:

```
W015 | <R file> | requirement 'R' is active in 2 configuration(s) but no TestCase covering it runs there: C1, C2
```

The rule **shall**:

- honour status suppression consistent with the flat uncovered-requirement check (draft `TestCase`s do not count as coverage; draft requirements are not flagged);
- be gateable via the severity-gating flags (e.g. `--deny W015`);
- **not** be emitted when the variability dimension is dormant — the existing flat uncovered-requirement check applies unchanged.

`W015` is a new, previously unused code (the spec's §9.11 already assigns `W010`–`W014`).

**Source:** Issue #10

**Acceptance criteria:** A `Configuration` activating a requirement with no covering in-config `TestCase` yields exactly one `W015` for R naming C (and every other configuration in which R is uncovered); full in-config coverage yields none; draft elements are suppressed; no `W015` is emitted when no feature model is linked.
