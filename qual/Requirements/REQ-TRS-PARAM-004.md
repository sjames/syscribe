---
id: REQ-TRS-PARAM-004
type: Requirement
title: Tool shall record and validate the binding time of FeatureDef parameters
status: draft
reqDomain: software
verificationMethod: test
---

A `FeatureDef` parameter ([[REQ-TRS-PARAM-001]], §9.7) **may** declare an optional `bindingTime:` field stating *when* in the product lifecycle the parameter's value is resolved. The value is one of the product-line-engineering binding-time triad, **ordered** earliest → latest:

| `bindingTime:` | Rank | Meaning |
|---|---|---|
| `compile` | 0 | fixed at build / code generation |
| `load` | 1 | fixed at deployment / installation / startup |
| `runtime` | 2 | bound dynamically while the system executes |

`bindingTime:` is orthogonal to `isFixed:`/`value:` (which express a value fixed in the model itself, i.e. *no* variability) — it constrains a parameter that genuinely varies. When a feature model is present, the tool **shall** enforce:

| Code | Severity | Command | Condition |
|---|---|---|---|
| `E230` | error | `validate` | A parameter declares `bindingTime:` whose value is not one of `compile`, `load`, `runtime`. |
| `E229` | error | `feature-check` | A parameter `P` is computed from a source `S` it depends on — a sibling named in `P`'s `derivedFrom:` expression, or the `bindTo:` target of `P` — and `rank(P) < rank(S)`, i.e. `P` would be bound **earlier** than a value it depends on. Checked only when **both** `P` and `S` declare a `bindingTime:`. |
| `W027` | warning | `validate` | A `Configuration`'s `parameterBindings` binds a parameter whose `bindingTime: runtime` — such a parameter is resolved by the running system, not at configuration time. Gateable (`--deny W027`). |

In addition, the `W017` "required parameter unbound" warning of [[REQ-TRS-PARAM-001]] **shall be suppressed** for a parameter whose `bindingTime: runtime`: a runtime parameter is legitimately left unbound by a `Configuration` because the running system supplies its value.

`E230` and `W027` and the `W017` suppression are emitted by `validate` (and, like the rest of [[REQ-TRS-PARAM-001]], by `feature-check`); the cross-parameter ordering check `E229` is a holistic check emitted by `feature-check` alongside the parameter-integrity rules of [[REQ-TRS-FM-003]].

**Source:** §9.7; PLE binding-time taxonomy (compile / load / run time).

**Acceptance criteria:**
- A parameter with `bindingTime: bogus` produces `E230` under `validate`; `compile`/`load`/`runtime` produce none.
- A `compile`-time parameter `derivedFrom` a `runtime` sibling (both with `bindingTime:`) produces exactly one `E229` under `feature-check`; the reverse ordering (`runtime` derived from `compile`) produces none.
- A `Configuration` that binds a `runtime` parameter produces `W027`.
- A selected feature with a required, unbound `runtime` parameter produces **no** `W017`; the same parameter at `compile`/`load` still produces `W017` when unbound.
