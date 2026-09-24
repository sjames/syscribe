---
id: REQ-TRS-PROJ-001
type: Requirement
name: Tool shall provide a --config lens that projects the model onto a configuration
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
- **Dormancy invariant:** when the model declares no `FeatureDef`, the variability dimension is dormant and a command run **without** `--config` behaves exactly as before.
- **`--config` on a dormant model:** an argument naming a stored `Configuration` (by id or qname — e.g. a MagicGrid parametric variant, which needs no `FeatureDef`) projects the identity (the whole model). Any other argument **shall** be a usage error (non-zero exit — `1`, or `validate`'s usage-error code `2`, exactly as for any other unresolvable `--config`) whose message states that the model declares no feature model and no such `Configuration`, rather than silently returning the whole model — which would hide a mistyped or misdirected argument. This applies to every command and MCP tool that accepts `--config`/`config`. (A `Baseline`'s `frozenScope.config` clause is a separate, stored scope and stays inert — REQ-TRS-BL-011.)
- A `--config` argument that does not resolve (unknown configuration id/qname, or an ad-hoc feature that is not a `FeatureDef`) **while a feature model exists** is a usage error (non-zero exit, clear message).
- Projection **shall** be deterministic.

**Source:** ADR-PROJ-001; §9.10 (projection).

**Acceptance criteria:** `list --config C` / `export --config C` over a model with `appliesWhen` show only the elements active in C; an ad-hoc `--config 'Features::A'` selects exactly the matching active set; on a model with no `FeatureDef`, omitting `--config` produces output identical to the pre-variability baseline, `--config X` naming no stored Configuration exits non-zero naming the missing feature model, for every command that accepts `--config`, while `--config` naming a stored Configuration still succeeds with the whole model; an unresolved `--config` argument with a feature model present errors with a non-zero exit.
