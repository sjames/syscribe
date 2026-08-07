---
type: ADR
id: ADR-SYS-PLANITEM-001
name: "Native PlanningItem: single-parent hierarchy, dual-form leaf evidence with per-entry waivers, GitHub-referenced status/type vocabulary"
status: accepted
tags:
  - planning
  - traceability
---

## Context

Syscribe models trace `Requirement → Architecture → Test`, but nothing in the model represents the
*work* of getting there — the day-to-day breakdown a team (or an LLM being guided through
development) actually executes, the shape a Jira epic/story/task or a GitHub issue hierarchy fills
today. That work currently lives entirely outside the model, in whatever external tracker a team
happens to use, disconnected from the traceability graph a `Requirement`'s `derivedFrom`/`satisfies`
already builds.

A concrete precedent for exactly this gap surfaced organically during the SysMLv2 submodel
feature's construction (`ADR-SYS-SYSMLV2-001`): a flat, session-scoped todo list
(`TaskCreate`/`TaskUpdate`/`TaskList`) drove a sequence of sub-agents through
implement → review → fix → verify → commit, repeated per requirement, with a human directing each
step. That list is ephemeral and un-versioned. `PlanningItem` is that same shape of thing made
durable, checked into git, and structurally part of the graph.

Two published lines of prior art were surveyed for the underlying mechanism:

- **Hierarchical Task Network (HTN) planning** — a task decomposes into subtasks until it bottoms
  out in *primitive actions*, which is the direct analogue of "each leaf node must result in some
  resultant evidence." Recent LLM-paired HTN work keeps decomposition symbolic and only invokes an
  LLM at decision points, which is the right posture for something meant to *guide* an LLM
  cheaply and auditably rather than re-plan from scratch at every step.
- **Graph of Thoughts** — exists specifically because a strict tree can't express *aggregation*
  (two subtasks feeding one piece of shared evidence, or one node with more than one parent). This
  was raised explicitly during scoping and **rejected**: see Decision 1.

## Decision

A new native, id-identified element type `PlanningItem` (`PI-*`), added to `ElementType` alongside
`Requirement`/`TestCase`/`ADR`/`Baseline` as a "dedicated handler" type — its own id pattern, its
own status/type vocabulary, its own validation rules — not a generic SysML usage.

Five sub-decisions, each with a rejected alternative:

1. **Strict single-parent tree, not a DAG.** A `PlanningItem` declares at most one `parent:`. A
   computed `children` reverse index mirrors `Requirement.derivedChildren`. *Rejected:* a DAG
   (multiple parents, shared/aggregated evidence) — the more theoretically complete structure per
   Graph-of-Thoughts, but explicitly decided against: it complicates the reverse index, the
   leaf-detection rule (a node needs *all* parents' completion states to reason about), and the
   mental model, for a capability not asked for. If convergent work becomes a real need later, it's
   an additive change (an optional second parent-like field), not a breaking one.
2. **A new `achieves:` field, not a reuse of `satisfies:`.** A `PlanningItem` with no `parent:`
   (top-level) must set `achieves: [<Requirement id-or-qname>, ...]` — "this branch of work exists to
   achieve these requirements' goals." *Rejected:* reusing the existing `satisfies:` field, which
   architecture elements already use to target a `Requirement`. `satisfies:` carries real, specific
   validation machinery (`E312`–`E315`'s domain-matching, no-parent-assignment, and leaf-satisfaction
   rules), all scoped to architecture semantics. Overloading it for planning-item purposes risks
   either silently inheriting rules that don't apply or requiring type-based carve-outs sprinkled
   through that machinery. A distinct field costs one new name and keeps both concerns clean.
3. **Evidence is a list of duck-typed entries — `ref:` (element) or `path:` (file/doc) — each with
   an optional per-entry `rationale:` that waives that entry's own check.** *Rejected:* a `type:`
   discriminator tag on each entry (`type: element` / `type: file`) — this codebase's existing
   `features:`-list convention (the `Allocation` form's `allocatedFrom`/`allocatedTo` pair) already
   establishes duck-typing by which keys are present as the idiom, not an explicit tag, and
   `ref:`/`path:` are already mutually self-describing. The waiver mechanism directly mirrors the
   established `ffiRationale` pattern (`REQ-TRS-*`'s HW/SW freedom-from-interference check): a
   rationale string co-located with the thing it excuses, not a separate global suppression list.
   `ref:` evidence is deliberately **not** restricted to a fixed allowed-kind list (architecture
   elements / `TestCase` only) — it accepts any element the resolver can find, matching this
   codebase's general permissive-then-validate posture; a closed list is easy to under-specify
   (an `ADR`, a `ReviewRecord`, an `Allocation` are all legitimate evidence) and costly to widen
   later.
4. **Status and item-type vocabularies are taken verbatim from GitHub's own current defaults, with
   one considered addition.** `status:` is `todo | in_progress | blocked | done` — GitHub Projects'
   three built-in defaults (`Todo`/`In Progress`/`Done`) plus `blocked`, added because an LLM being
   guided step-by-step needs to distinguish "not started" from "can't proceed" and GitHub's bare
   default doesn't. `itemType:` is `bug | task | feature` — exactly GitHub's own current default
   Issue Types, unmodified. *Rejected:* a richer traditional PM taxonomy (`epic`/`spike`/`chore`
   added) — closer to some agile conventions, but the point of referencing GitHub specifically was
   fidelity to a vocabulary already familiar to anyone who's used GitHub Issues, not maximal
   expressiveness; the vocabulary is a plain string field, not a closed Rust enum, so extending it
   later needs no schema migration.
5. **Pure replacement — no `extRef` integration, no sync.** A `PlanningItem` does not read from or
   write to an external Jira/GitHub issue. *Rejected:* using the already-existing generic `extRef:`
   field for provenance linkage, or a bidirectional sync engine — both left for a genuinely separate
   future effort if ever needed; this feature's job is to be the source of truth outright, not to
   mirror one.

`appliesWhen:` — product-line gating — needs **no new decision or mechanism at all**. It is already
a universal, type-agnostic frontmatter field: `feature_model.rs`/`projection.rs`/`validator.rs`'s
`E209` check all read it off any `RawElement` with zero type filtering (confirmed independently
during the SysMLv2 `@SyscribeFeature` work, `ADR-SYS-SYSMLV2-001`). A `PlanningItem` implementing a
product-line feature simply sets `appliesWhen:` like any other element; `feature-check --deep`,
`projection`, and `configure` treat it identically to a native gate with zero solver changes.

## Rationale

- **Why is this in-scope for the traceability graph at all, rather than staying an external
  tracker concern?** Because "guide an LLM step-by-step through development" needs the guidance
  itself — what's next, what's blocked, what counts as proof of done — to be resolvable the same
  way every other cross-reference in this model resolves: by id/qname, checked by the same
  validator, visible in the same web UI and MCP surface. An external tracker can't participate in
  `derivedFrom`/`satisfies`/`verifies` resolution; a native element can.
- **Why single-parent over the more general DAG the research surfaced?** Simplicity that matches
  an existing, already-battle-tested precedent (`Requirement.derivedFrom` + `derivedChildren`) beats
  generality nobody asked for. The single genuinely compelling DAG use case — one piece of evidence
  serving two different leaves — is still expressible today: nothing stops two different
  `PlanningItem`s from independently listing the same `ref:`/`path:` evidence entry: that's evidence
  *reuse* by value, not a graph-structural convergence, and it needs no DAG at all.
- **Why hold this to schema + validation only, no MCP tools yet?** The MCP server's existing
  guarded-write surface (`WRITE_TOOLS`: `create_element`/`update_element`/`move_element`/
  `delete_element`/`apply_changes`/`suspect_accept`) already lets any element type, including a new
  one, be created/updated/moved/deleted with zero new code — a `PlanningItem` gets a working,
  guarded write path for free the moment it's a recognised `ElementType`. A `next_actionable_item`-
  style tool that actively drives an agent loop is a materially different, larger piece of work
  (closer to what LangGraph does at runtime) and is deliberately deferred until the schema itself
  has been used and proven.

## Addendum: `blockedBy:` (REQ-TRS-PLANITEM-007)

`REQ-TRS-PLANITEM-000` already named "dependency" as something this feature exists to make
resolvable in-graph, alongside breakdown and completion evidence, but no field ever implemented
it — `status: blocked` could be declared with nothing saying *what* it's blocked on. Found and
closed as a direct follow-on, not a course correction: a `blockedBy:` field, resolved exactly like
`evidence.ref:` (any element, unrestricted by kind — an undecided `ADR` or an external dependency
is as legitimate a blocker as another `PlanningItem`), with a cycle-detection posture matching
`parent:`. Deliberately **not** required to be non-empty when `status: blocked` (unlike
`evidence:` on a `done` leaf) — being blocked is a transient working state that needs no proof,
only completion does; a stale `blockedBy:` left over after `status:` moves on is instead a warning,
not an error, since the two fields are independently author-maintained.

## Consequences

- A model with no `PlanningItem` elements is completely unaffected.
- The leaf-evidence rule (`REQ-TRS-PLANITEM-006`) is severity-graded by status, not a blanket leaf
  check: a leaf `PlanningItem` not yet `status: done` raises nothing (evidence is proof of
  completion, not a prerequisite for starting); once `status: done`, at least one non-waived
  evidence entry is required, or it's an error — claiming done with no proof is a correctness
  defect, not a soft, time-bound gap the way an unassigned-but-still-`approved` leaf `Requirement`
  is (`W300`).
- **Explicitly out of scope**, tracked as follow-on only if a concrete need arises: MCP tools that
  actively drive an LLM through a `PlanningItem` graph (`next_actionable_item`, `mark_evidence`,
  etc.), a formal state-machine-backed status model (reusing the existing HSM feature instead of a
  plain string enum), and any Jira/GitHub sync or `extRef` convention specific to planning items.
