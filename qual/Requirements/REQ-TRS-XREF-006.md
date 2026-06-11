---
id: REQ-TRS-XREF-006
type: Requirement
title: "Tool shall hint when an unresolved cross-reference wrongly includes the model-root package name"
status: draft
reqDomain: software
verificationMethod: test
---

A qualified name is derived from a file's path **relative to the model root**, and the root
package (the root `_index.md`) contributes **no** segment — so the root package's `name:` is
**not** part of any qualified name (cross-references start at the first sub-namespace, e.g.
`ProblemDomain::BlackBox::Foo`, never `RootName::ProblemDomain::…`). Authors — humans and
LLMs — routinely prefix the root package name and then get only a generic "does not resolve"
error that does not explain the real cause. This requirement turns that into a targeted,
self-correcting hint, and documents the rule.

### Behaviour

- When a cross-reference fails to resolve (any of the existing unresolved-reference findings
  — e.g. `E102`/`E103`/`E311`/`E316`/`E502`/`E503` and the generic supertype/typedBy/subsets/
  redefines/connection resolution errors), the tool **shall** test whether the offending
  reference string **begins with the model-root package's `name:` followed by `::`** and,
  after stripping that prefix, **resolves** to a known element.
- If and only if the stripped form resolves, the tool **shall** append an advisory hint to
  the existing finding, naming the corrected reference — e.g.
  `hint: the model-root package name is not part of qualified names; did you mean 'ProblemDomain::BlackBox::Foo'?`.
- This is **diagnostic only**: it adds explanatory text to the existing finding. It introduces
  **no new code**, does **not** change resolution behaviour (the reference still does not
  resolve and the original error still fires), and never auto-rewrites the model.
- The hint **shall not** fire when stripping the prefix does not produce a resolvable
  reference (avoids misleading suggestions), nor when the model root package has no `name:`.

### Documentation

- The format specification **shall** state explicitly that the model-root package `name:` is
  not part of qualified names (qualified names are path-relative and the root contributes no
  segment), alongside the existing qualified-name derivation rules (§11.3 / naming rules).

**Source:** developer-experience friction identified while building the `model_mg/` MagicGrid
model — a root `_index.md` named `EVChargingStation` led to `EVChargingStation::…` cross-refs
that failed with opaque resolution errors. Refines the qualified-name resolution behaviour of
[[REQ-TRS-XREF-005]] (§11.3, §11.10).

**Acceptance criteria:**

- A `derivedFrom:`/`refines:`/`supertype:` reference written as `<RootName>::A::B`, where
  `A::B` is a real element, still raises its normal unresolved-reference error **and** carries
  a hint naming `A::B` as the intended target.
- The same reference written correctly as `A::B` resolves with no finding and no hint.
- An unresolved reference that does **not** start with the root package name, or whose stripped
  form still does not resolve, produces the normal error with **no** hint.
- The specification documents that the model-root package name is excluded from qualified names.
