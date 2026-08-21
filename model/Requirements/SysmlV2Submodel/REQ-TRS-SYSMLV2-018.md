---
type: Requirement
id: REQ-TRS-SYSMLV2-018
name: "A SysMLv2 state def/state maps to the native StateDef/State schema — subStates, transitions with guard/accept/effect, entry/do/exit action names, isInitial/isFinal"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - state-machines
---

A `state def`/`state` usage shall be synthesized into a native `StateDef`/`State` element carrying
the same `subStates:`/`transitions:`/`entryAction:`/`doAction:`/`exitAction:` shape a hand-authored
one uses (`docs/model-guide/state-machines.md`'s canonical schema), so a SysMLv2-authored state
machine participates fully in the existing `W070`–`W080` completeness checks and in `satisfies:`/
`verifies:` traceability, with zero validator changes. A top-level `state`/`state def` (declared
directly in a package or part) is its own real, qname-addressable element; a `state` nested inside
another `StateDef`/`StateUsage`'s own body becomes an inline `subStates:` entry only, never a
separate element — matching how a hand-authored composite state machine is already written.

## Rationale

`StateDef`/`State` already carry full, tested `satisfies:`/`verifies:` traceability participation
for hand-authored elements (this session's `satisfies:`-shape audit, mirroring `refines:`'s `E316`/
`REQ-TRS-MG-010` precedent). Without this requirement, a SysMLv2-authored state machine is invisible
to the graph entirely — it cannot be `satisfies:`-linked, `verifies:`-targeted, or browsed — leaving
that traceability capability asymmetric between hand-authored and SysMLv2-authored content for no
reason tied to the state-machine feature itself. The target schema is not invented for this
requirement: it is the schema `model/Behavior/FlightStates.md` already uses and
`docs/model-guide/state-machines.md` already documents.

## Scope

- `StateDef`/`StateUsage` share one grammar body (`StateDefBody`/`StateDefBodyElement`) at every
  nesting depth; mapping recurses through nested `StateUsage` children (each becoming an inline
  `subStates:` entry, itself recursively carrying its own `subStates:`/`transitions:`), applying
  `isInitial:`/`isFinal:` from sibling `then <name>;`/`final <name>;` statements by name-match.
- `transitions:` uses the canonical `source`/`target`/`accept`/`guard`/`effect` vocabulary only
  (never the deprecated `from`/`to`/`trigger` aliases, so `W075` never fires on synthesized output).
  A transition nested inside a specific substate's own body omits `source:` (that substate's own
  `name:` supplies the implicit source, per the canonical schema's own documented equivalence); a
  transition at the composite's own top level requires an explicit `source:` and is dropped
  otherwise (a source-less top-level transition means nothing).
- `effect:` matches `validator.rs::collect_state_refs`'s exact `W079` resolution contract: a
  `Perform`/`Accept`/`Send` effect with a real target type becomes `{name, typedBy}`
  (`W079`-checked); one with only a local action name becomes `{name}` only (not checked, avoiding
  a spurious false positive on what isn't necessarily a global qname).
- `entryAction:`/`doAction:`/`exitAction:` carry only the referenced action's own name — the nested
  `body:` grammar `EntryAction`/`DoAction`/`ExitAction` carry (typed as *state*-body grammar, a
  parser-leniency artifact, not real action-body semantics) has no matching native field and is not
  mapped. `isParallel:` is not representable at all in this parser version — neither `StateDef` nor
  `StateUsage` carries a parallel/orthogonal-region flag — and stays unset.
- Guard/condition text for `Expression` shapes beyond common references, literals, and binary/unary
  operators falls back to a fixed, kind-naming placeholder rather than vanishing (a guard must never
  disappear, since `W072`'s non-determinism check and the canonical schema both depend only on the
  field being present) — a Syscribe-owned, revisitable-later rendering limitation, not a semantic
  claim about the guard's true meaning.
- Out of scope, unaffected by this requirement: `exhibitsStates:` (a `Part`-level field naming a
  `StateDef` it exhibits — never synthesized by SysMLv2 ingestion either before or after this
  requirement, per `REQ-TRS-SYSMLV2-017`'s own note).

**Acceptance criteria:** a package-wrapped `state def`/`state` synthesizes a real `StateDef`/`State`
element with `subStates:`/`transitions:` matching the documented canonical schema; the existing
`W070`–`W080` checks fire on a deliberately-broken synthesized fixture (dead state, trap state,
non-deterministic transition pair, unresolved `effect`) exactly as they would on an equivalent
hand-authored one, with no `validator.rs` changes required; a genuinely still-unmapped construct
(e.g. `calc def`) remains invisible, unaffected.
