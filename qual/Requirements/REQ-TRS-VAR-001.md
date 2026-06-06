---
id: REQ-TRS-VAR-001
type: Requirement
name: Variability Is Opt-In
title: Tool shall treat variability as dormant unless a feature model is linked
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** activate the variability dimension only when **both** of the following hold:

1. at least one `FeatureDef` element exists, **and**
2. at least one element links to it — a `Configuration` selecting features, and/or any element (including a `TestCase`) carrying `appliesWhen:`.

When the variability dimension is **dormant**, the tool **shall**:

- produce `validate` output that is byte-identical to the same model with no PLE elements present (no new errors or warnings);
- treat `appliesWhen:` on an element as **inert** (ignored), *except* that an unresolved or wrong-type reference is still reported as `E209`;
- for variant-only commands (`matrix`, per-`Configuration` coverage), print a clear "no feature model present" notice and fall back to the flat requirement↔testcase view **without error**.

When the variability dimension is **active**, an element with no `appliesWhen:` **shall** be treated as *always active* (unconditional). This rule applies uniformly to requirements and `TestCase`s.

**Source:** Issue #7

**Acceptance criteria:** A model with zero `FeatureDef` validates byte-identically to the pre-variability baseline; adding `FeatureDef` + `Configuration` elements but leaving requirements without `appliesWhen:` yields a matrix in which every requirement is active in every `Configuration` column (no spurious N/A); a typo'd `appliesWhen:` is `E209` in all modes, while an inert `appliesWhen:` with no feature model produces no finding.
