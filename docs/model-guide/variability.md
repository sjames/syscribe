# Variability & Product Lines

`GUIDE · VARIABILITY`

Syscribe models a whole **product line** as one repository — the **150% model** — and projects it onto individual products (**100% models**) on demand. Everything in this guide is **opt-in**: a model with no `FeatureDef` behaves exactly as a single-product model, and none of these rules or commands change its output.

The capability has four layers, each building on the last:

1. **Feature model** — what can vary (`FeatureDef`, `Configuration`).
2. **Conditioning** — which elements belong to which variants (`appliesWhen:`).
3. **Analysis** — is the variability sound? (`feature-check`, `feature-check --deep`).
4. **Projection** — view and validate one variant (`--config`).

---

## 1. The feature model

A **`FeatureDef`** is one node of the feature tree — a selectable characteristic. Nesting (directory or `parentFeature:`) forms the tree; `groupKind:` gives the variability type.

```yaml
# Features/Platform/_index.md        (a mandatory XOR group)
---
type: FeatureDef
id: FEAT-PLATFORM-001       # mandatory stable id (E201 if missing)
name: Platform
mandatory: true             # membership: every product has a platform...
groupKind: alternative      # ...and picks exactly one child (XOR)
---
# Features/Platform/CortexM.md
---
type: FeatureDef
id: FEAT-CORTEXM-001
name: CortexM
groupKind: optional
---
```

Two orthogonal axes (`ADR-FM-003`):

- **`groupKind`** describes how a feature's **children** are grouped: `optional` · `alternative` (XOR) · `or`.
- **`mandatory: true`** describes the feature's **membership** relative to its parent — selected whenever the parent is (or always, at top level). It is independent of `groupKind`, so a node can be a *mandatory XOR group* as above. (The legacy `groupKind: mandatory` is a shorthand for `mandatory: true` on a leaf.)

Cross-tree constraints use `requires:` / `excludes:` (qualified names of other features). Quantitative variability uses typed `parameters:` (see below).

A **`Configuration`** (id `CONF-*`) is a complete named product: a `features:` **map** of `<FeatureDef qname>: true/false`.

```yaml
---
type: Configuration
id: CONF-MPS2-WDT-001
title: "MPS2 board with watchdog"
status: approved
featureModel: Features
features:
  Features::Platform::CortexM: true
  Features::Platform::RiscV: false
  Features::Wdt: true
---
```

> **Footgun guard (`W016`):** the selection block must be the `features:` **map**. A legacy `selections:` list is ignored and flagged — `syscribe show <CONF>` displays the parsed selections so you can see exactly what was read.

---

## 2. Conditioning elements — `appliesWhen:`

`appliesWhen:` conditions **any** element (including a `TestCase`) on a boolean expression over `FeatureDef` qualified names:

```yaml
appliesWhen: Features::Wdt                                  # bare
appliesWhen: [Features::Wdt, Features::CortexM]             # list = AND
appliesWhen: "Features::CortexM and Features::Mpu"          # expression
appliesWhen: "(Features::A or Features::B) and not Features::C"
```

An element with no `appliesWhen:` is **always active**. Every operand must resolve to a `FeatureDef` (else `E209`). There is **no `runsIn` field** — a `TestCase` runs in a configuration iff its `appliesWhen:` is satisfied by that configuration's selections.

### Gating a whole subtree — package-level `appliesWhen:`

Put `appliesWhen:` on a **`Package`** (`_index.md`) to condition its entire subtree at once — ideal for enabling/disabling a cohesive variant of requirements + architecture + tests together:

```yaml
# Delivery/_index.md
type: Package
appliesWhen: Features::Payload::Delivery
```

Every element under `Delivery/` inherits that condition. An element's **effective condition** is its own `appliesWhen:` if it has one, else the nearest ancestor package's, else always-active — never a combination. To keep that unambiguous, **at most one** node per path may declare `appliesWhen:`:

- a nested declaration (element or sub-package under a gated package) → **`E228`**;
- `appliesWhen:` on a `FeatureDef`/`Configuration`, on a package whose subtree contains one, or on the model root → **`E228`**;
- a gated package with an empty subtree → **`W026`**.

Because the subtree moves together, references *inside* it never escape; only references from outside into the gated subtree are flagged when the condition is off. Use package-level gating for all-or-nothing subtrees; keep element-level `appliesWhen:` (with the enclosing package **not** gated) when you need a strict subset. `why-active <el> --config <C>` shows when a condition is inherited from a package.

### Quantitative variability — feature parameters

A `FeatureDef` may declare typed `parameters:`; a `Configuration` binds them under `parameterBindings:`:

```yaml
# FeatureDef
parameters:
  - { name: motorKV, type: ScalarValues::Real, range: "900..=1200", isRequired: true }
# Configuration
parameterBindings:
  Features::Motor.motorKV: 1050.0      # canonical reference: <Feature qname>.<param>
```

A **parameter reference** is always the dotted form `Features::Path::Feature.param` — `::` between feature segments, a single `.` before the parameter member. The same form is used in `parameterBindings:` keys, `parameterConstraints` expressions, and `bindTo:` targets. `range:` accepts `"min..max"` and the inclusive `"min..=max"`.

Binding rules (run by `validate`): bind a parameter of an unselected feature (`E203`), bind a fixed parameter (`E204`), out of `range:` (`E205`), not in `enumValues:` (`E206`), unresolved/legacy-`::` path (`E222`); a required, unbound parameter warns (`W017`).

**Binding time.** A parameter may declare an optional `bindingTime:` — *when* its value is resolved, from the PLE triad ordered earliest→latest: `compile` (build / codegen) · `load` (deployment / startup) · `runtime` (live). It is orthogonal to `isFixed:`/`value:` (a value fixed in the model, i.e. no variability). Absent = unspecified (opts out of the checks below).

```yaml
parameters:
  - { name: motorKV, type: ScalarValues::Real, range: "900..=1200", bindingTime: load }
```

Rules: an unrecognised value is `E230` (`validate`); a parameter that binds **earlier** than a `derivedFrom`/`bindTo` source it depends on is `E229` (`feature-check`, checked only when both ends declare a `bindingTime:`); binding a `runtime` parameter in a `Configuration` warns `W027` (`validate`), and `W017` is suppressed for an unbound `runtime` parameter.

**Cross-feature constraints.** A package `_index.md` may declare `parameterConstraints:` — numeric couplings evaluated by `feature-check` against every applicable `Configuration`:

```yaml
parameterConstraints:
  - id: PC-AMP-MIN
    expression: "Features::Topology.maxCpus >= 2"        # comparison over dotted refs
    appliesWhen: "Features::Cpu::CortexA and Features::Topology::Amp"   # boolean predicate
    severity: error        # violation -> E221 (or W025 when severity: warning)
```

A violation in a configuration whose `appliesWhen:` holds is `E221` (or `W025` for `severity: warning`); an unresolved parameter path is `E213`; an `appliesWhen:` feature selected in no configuration is `W014`.

---

## 3. Analysis

### The coverage matrix

```bash
syscribe -m model/ matrix            # Requirement × Configuration grid
syscribe -m model/ matrix --json --tag safety
```

Rows are requirements, columns are `Configuration` elements; cells are **covered** (`✓`), **gap** (`✗`), or **N/A** (`—`, requirement not active in that variant). `W015` turns a per-configuration gap into a gateable finding.

### `feature-check` — holistic feature-model validation

Run separately from `validate` (it does not run on every `validate`):

```bash
syscribe -m model/ feature-check
```

| Rule | Meaning |
|---|---|
| `E212` | `requires`/`excludes` target is not a `FeatureDef` |
| `E219` / `E220` | a selected feature's `requires` unmet / `excludes` violated in a `Configuration` |
| `W011` / `W012` | optional feature selected in no / every `Configuration` |
| `E207` | circular `derivedFrom:` among a feature's parameters |
| `E202` | a `bindTo:`-propagated value is outside the component parameter's `range:` |
| `E229` | a parameter's `bindingTime:` is earlier than a `derivedFrom`/`bindTo` source it depends on |
| `E213` / `W014` | `parameterConstraints` unresolved path / `appliesWhen` feature used in no config |
| `E221` / `W025` | `parameterConstraints` expression evaluates false for an applicable `Configuration` (`W025` when `severity: warning`) |
| `W024` | **orphan feature** — referenced by no `appliesWhen:` and selected by no `Configuration` (gates nothing, ships in nothing); gate with `--deny W024` |

### `feature-check --deep` — SAT-backed whole-space analysis

Encodes the Boolean feature layer and reasons over **all** valid configurations (deterministic, pure-Rust [batsat], comfortably ~500 features; see `ADR-FM-002`):

| Result | Code |
|---|---|
| **void** model (no valid configuration) | `E223` (with a minimal conflict explanation + diagnoses) |
| **dead feature** (selectable in no configuration) | `E224` |
| **core feature** (in every configuration) | reported |
| **false-optional** (forced whenever its parent is) | `W018` |
| **invalid `Configuration`** (full group/cardinality semantics) | `E225` |
| **dead element** (`appliesWhen` unsatisfiable) | `W021` |
| **aggregate coverage** (active in some config, covered in none) | `W022` |

Companion commands:

```bash
syscribe -m model/ feature-check --count        # number of valid configurations
syscribe -m model/ feature-check --enumerate    # list them
syscribe -m model/ feature-check --deep --prove <dir>   # DIMACS CNF of each UNSAT finding
syscribe -m model/ configure <Configuration>    # partial selection → forced/free features
```

### Discoverability

Four read-only commands answer "what can vary, what does each feature gate, and why is this element in this product?"

```bash
syscribe -m model/ features                              # the feature model as a tree
syscribe -m model/ feature Features::Payload::Delivery   # one feature's card
syscribe -m model/ matrix --features                     # Feature × Configuration grid (the product map)
syscribe -m model/ list PartDef --feature Features::DualFlightController   # elements a feature gates
syscribe -m model/ why-active <element> --config CONF-X  # is this element active here, and why?
```

- **`features`** — the feature tree: each node's `groupKind`, `requires`/`excludes`, parameters, and a *selected in N/M configs* rollup.
- **`feature <qname>`** — one feature's card: its doc, group, constraints, parameters, the configurations that select it, and every element it **gates** (whose `appliesWhen:` names it).
- **`matrix --features`** — which feature ships in which product.
- **`list <type> --feature <F>`** — the elements gated by `F` (orthogonal to `--tag` and `--config`).
- **`why-active <element> --config <C>`** — prints the element's `appliesWhen:`, the config's relevant selections, and a `Verdict:` of `active` / `inactive` / `always active`.

---

## 4. Projection — the `--config` lens

The repository is the 150% superset; `--config` projects it onto one variant and runs the command over only the **active** elements. The argument is a stored `Configuration` (id/qname) or an ad-hoc feature set (`'Features::A,Features::B'`).

```bash
syscribe -m model/ list Requirement --config CONF-UAV-MAPPING-001
syscribe -m model/ export --config 'Features::Propulsion::Hex,Features::Payload::Delivery' --json
syscribe -m model/ validate --config CONF-UAV-DELIVERY-001    # certify THIS product
syscribe -m model/ validate --all-configs                     # CI gate over every product
syscribe -m model/ diff --config CONF-UAV-SURVEY-001 \
                        --config CONF-UAV-DELIVERY-001         # what differs between products
```

> The bundled `model/` is a runnable UAV product line — every command on this page works against it. See the [worked example](index.md#worked-example-the-uav-product-line) for its shape.

`validate --config` re-runs the full validation in the lens (coverage, §12 traceability, safety) **and** flags **escaping references** — an active element pointing at one inactive in the variant:

- **structural** escape (`typedBy`/`supertype`/`subsets`/connection/allocation) → `E226` (error — the variant is broken);
- **traceability** escape (`verifies`/`satisfies`/`derivedFrom`/`breakdownAdr`) → `W019` (warning).

`feature-check --deep` proves the same can't happen in **any** valid configuration: for each reference edge it checks `appliesWhen(X) ⇒ appliesWhen(Y)` via SAT and reports a violable structural edge as `E227` (with a witness selection), a traceability edge as `W020`.

---

## Qualification angle

Safety standards certify a **product**, not a superset. The 150% model is the reusable asset; `validate --config <C>` is the per-variant evidence — *this product's* requirements are covered by *this product's* tests and satisfied by *this product's* architecture. `validate --all-configs` makes that a CI gate across the family.

## Further reading

- [CLI Reference](../cli/index.md) — all flags and examples
- [Rule Reference](../validation/rules.md) — every code (E2xx PLE, projection, deep)
- Format spec §9 (`syscribe spec validation`, `syscribe spec fields`)
