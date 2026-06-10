---
id: REQ-TRS-SAFE-008
type: Requirement
name: GSN argument layer (Argument, AssumptionOfUse) and safety-case view
title: Tool shall represent the GSN safety-argument layer (Argument, AssumptionOfUse) and render a safety-case (GSN) view
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** close the remaining gap of GitHub issue #20: the structured
**safety-argument layer** (Goal Structuring Notation, GSN) and a navigable
**safety-case view**.

> **Pre-existing scope (issue #20).** The hazard-and-goal layer already exists:
> `SafetyGoal` (SG-*), the issue's "Hazard" (`HazardousEvent`, HE-*), and the
> FMEA "FailureMode" layer (`FMEASheet`/`FMEAEntry`). This requirement adds **only**
> the argument-layer types `Argument` and `AssumptionOfUse`, plus the
> `safety-case` view that ties goals → arguments → evidence together.

## Part 1 — `Argument` type (GSN node)

The tool **shall** recognise a new native element `type: Argument` carrying a stable
opaque id matching `^ARG(-[A-Z0-9]{2,12})+-[0-9]{3,8}$` (`ARG-*`). The id is a valid
cross-reference target and is added to `is_stable_id` (regex `arg_re`, predicate
`is_arg_id`). Fields:

- `argumentType:` (string, `RawFrontmatter.argument_type`) ∈ { `claim`, `strategy`,
  `solution` }. Absent → treated as `claim`.
- `supports:` (string or list, `RawFrontmatter.supports`) — the `SafetyGoal` or parent
  `Argument` this node argues for (the GSN supported goal). Each ref resolves via the
  `Resolver`.
- `evidence:` (string or list, `RawFrontmatter.evidence`) — refs to supporting
  `Requirement` / `TestCase` / sub-`Argument` / `AssumptionOfUse` (the GSN children).
  Each ref resolves via the `Resolver`.
- `id:` / `title:` / `status:` (existing common fields).

| Code | Severity | Condition |
|---|---|---|
| `E852` | error | `Argument` is missing `id`, `title`, or `status`. |
| `E853` | error | `Argument.id` does not match the `ARG-*` pattern. |
| `E854` | error | `Argument.argumentType` is not one of `claim`, `strategy`, `solution`. |
| `E855` | error | An `Argument.supports` or `Argument.evidence` ref does not resolve to any model element. |
| `W040` | warning | A `claim`/`strategy` `Argument` has **both** an empty `supports` **and** an empty `evidence` (an orphan GSN node arguing nothing). |

## Part 2 — `AssumptionOfUse` type (SRAC)

The tool **shall** recognise a new native element `type: AssumptionOfUse` (a
safety-related application condition / SRAC) with a stable opaque id matching
`^AOU(-[A-Z0-9]{2,12})+-[0-9]{3,8}$` (`AOU-*`). The id is added to `is_stable_id`
(regex `aou_re`, predicate `is_aou_id`). Fields:

- `id:` / `title:` / `status:` (existing common fields).
- `appliesTo:` (string or list, `RawFrontmatter.applies_to`) — the `SafetyGoal` /
  `Argument` / `Requirement` it constrains. Each ref resolves via the `Resolver`.

| Code | Severity | Condition |
|---|---|---|
| `E856` | error | `AssumptionOfUse` is missing `id`, `title`, or `status`. |
| `E857` | error | `AssumptionOfUse.id` does not match the `AOU-*` pattern. |
| `E858` | error | An `AssumptionOfUse.appliesTo` ref does not resolve to any model element. |

## Part 3 — `safety-case` view

The tool **shall** provide a read-only `safety-case` subcommand:

```
syscribe -m <root> safety-case [<SG-id>] [--json]
```

For each top `SafetyGoal` (or only the one named by `<SG-id>`), the view **shall**
render the GSN argument tree:

- the Goal, then the `Argument`s whose `supports` names it, then each Argument's
  `evidence` (recursing into sub-`Argument`s; `Requirement`s and `TestCase`s appear as
  leaves). For a `TestCase` leaf the view **shall** show the ingested verdict
  (`pass` / `fail` / `unknown`) when a results sidecar is present, reusing the
  `tc_verdict` helper.
- any `AssumptionOfUse` whose `appliesTo` names the goal or an argument under it.

The tool **shall also fold in the implicit chain** that exists even without explicit
`Argument` nodes: `SafetyGoal ← Requirement` (via `derivedFromSafetyGoal`) `← TestCase`
(via `verifies`). This makes the view useful on models that have goals + requirements +
tests but no `Argument` nodes yet (the bundled `model_sil/`).

Output:

- **text** — a hand-rolled indented GSN-style tree using `├──` / `└──` connectors,
  each node kind-prefixed, e.g. `[strategy]`, `[claim]`, `[solution]`,
  `[evidence:Requirement]`, `[evidence:TestCase]`, `[AoU]`.
- **`--json`** — `{ "goals": [ { "id", "title", "arguments": [ ...nested... ],
  "requirements": [ ... ], "assumptions": [ ... ] } ] }`, valid JSON.

The subcommand **shall** be read-only (reuses `Resolver`) and exit 0. With no
`SafetyGoal` in the model it **shall** print a notice and exit 0.

## Opt-in / non-regression

The two new types are purely additive. A model with no `Argument` or `AssumptionOfUse`
element emits **no** new findings, so the bundled demo models (`model/`, `model_auto/`,
`model_sil/`) keep their existing `validate` exit codes unchanged.

## Acceptance criteria

1. `type: Argument` and `type: AssumptionOfUse` parse; `syscribe spec types` lists both.
2. A valid GSN model (goal + strategy Argument + sub-Argument/solution + AoU) validates
   with **no errors**.
3. An unresolved `Argument.supports` yields `E855`; an orphan `claim` Argument yields
   `W040`.
4. `safety-case` text shows the goal with its strategy/claim/evidence and the AoU;
   `safety-case --json` is valid JSON exposing `goals[].arguments` and `assumptions`.
5. `syscribe -m model_sil safety-case` shows the existing
   `SafetyGoal → Requirement → TestCase` chains via the implicit fold-in.
6. Bundled-model `validate` exit codes are unchanged.
