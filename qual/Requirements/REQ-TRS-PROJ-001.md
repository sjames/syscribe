---
id: REQ-TRS-PROJ-001
type: Requirement
title: Tool shall provide a --config lens that projects the model onto a configuration
status: draft
reqDomain: software
verificationMethod: test
---

The repository is a **150% model** describing the whole product line; a `Configuration` projects it onto a **100% model** (one variant). The tool **shall** expose this projection as a composable **`--config` lens**.

### Projection

- An element is **active** in a selection iff its `appliesWhen:` holds for that selection (an element with no `appliesWhen:` is always active). This is the same predicate used by `matrix` and `W015`.
- `project(model, selection)` is the set of active elements; it is itself a valid Syscribe model, so the lens **shall** be implemented by filtering to the active subset and reusing the existing command for that subset.

### `--config <arg>`

The lens flag **shall** be accepted on the read/validation commands: `validate`, `trace`, `why`, `who-verifies`, `list`, `refs`, `links`, `export`, `diagram`. Its argument resolves to a **selection** in either form:

| Form | Meaning |
|---|---|
| `<CONF-id>` or `<qname>` | A stored `Configuration`; its `features:` map is the selection (closed-world: a feature absent from the map is deselected) |
| `Features::A,Features::B,…` | An **ad-hoc** feature set: the listed `FeatureDef`s are selected, all others deselected (for what-if exploration; pairs with `configure`) |

### Behaviour

- With `--config`, a command **shall** operate only over the active subset; its output reflects only active elements.
- **Dormancy invariant:** when the model declares no `FeatureDef`, the variability dimension is dormant and `--config` is **inert** — the command behaves exactly as without it.
- A `--config` argument that does not resolve (unknown configuration id/qname, or an ad-hoc feature that is not a `FeatureDef`) **while a feature model exists** is a usage error (non-zero exit, clear message).
- Projection **shall** be deterministic.

**Source:** ADR-PROJ-001; §9.10 (projection).

**Acceptance criteria:** `list --config C` / `export --config C` over a model with `appliesWhen` show only the elements active in C; an ad-hoc `--config 'Features::A'` selects exactly the matching active set; on a model with no `FeatureDef`, `--config X` produces output identical to omitting the flag (dormant fallback); an unresolved `--config` argument with a feature model present errors with a non-zero exit.
