---
id: REQ-TRS-SM-004
type: Requirement
name: Tool shall validate parallel state machines per region (region-aware W070–W074)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise a **parallel (orthogonal) state machine** — a `StateDef` /
`State` with `isParallel: true` — per SysMLv2 §7.18 (`temp/sysml2_spec.pdf`): its **direct
substates are concurrent regions**, each itself a (composite) region holding its own
`subStates:` and transitions. The substates of a parallel state run concurrently.

For each region (each direct substate that declares its own `subStates:`), the tool
**shall** apply the §22.1 completeness checks **scoped to that region**, over the region's
own substate roster and the edges produced by the canonical extractor of
[[REQ-TRS-SM-001]] for that region (its substates' nested transitions plus the region's own
`transitions:`):

- `W073` (missing initial) and `W074` (multiple initial) **shall** be evaluated **per
  region** — every region needs exactly one `isInitial: true` substate, not the parallel
  parent.
- `W070` (dead) and `W071` (trap) **shall** apply within each **flat** region; a region
  whose own substates are further composite defers those two to the hierarchy phase.
- A finding raised inside a region **shall** name the region for diagnosis.

A parallel state itself has **no** single top-level initial substate, so the flat
single-region `W073`/`W074` **shall not** be applied to the parallel parent directly.

**Source:** SysMLv2 §7.18 (parallel states); region-aware refinement of the flat checks
[[REQ-TRS-SM-003]]. Cross-region and arity rules are [[REQ-TRS-SM-005]].

**Acceptance criteria:**

- A parallel machine whose every region has exactly one initial and is connected raises no
  W070–W074.
- A parallel machine with a region that has no `isInitial` raises `W073` naming that region;
  a region with two initials raises `W074`.
- A dead/trap substate **inside** a flat region is flagged with the region named.
- The flat single-region `W073` is **not** raised against the parallel parent itself.
