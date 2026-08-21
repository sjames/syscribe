---
type: Requirement
id: REQ-TRS-SYSMLV2-019
name: "A SysMLv2 action def/action maps to the native ActionDef/Action schema — subActions, controlNodes, successionConnections, real if/while/loop/for recursion, fork/join/decide/merge as name-only control nodes"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - behavior
---

An `action def`/`action` usage shall be synthesized into a native `ActionDef`/`Action` element
carrying the same `subActions:`/`controlNodes:`/`successionConnections:` shape a hand-authored one
uses (`model/Behavior/{TakeoffAction,LandingAction,WaypointNavAction,MissionExecution}.md`'s
existing convention), going as deep as the pinned parser (`sysml-v2-parser = "0.54.0"`) actually
retains: `if`/`while`/`loop`/`for` recurse for real; `fork`/`join`/`decide`/`merge` become flat,
name-only control nodes with no recoverable internal content, since the parser itself discards their
block-body contents. A top-level `action`/`action def` is its own real, qname-addressable element;
an action-body construct found nested inside another `ActionDef`/`ActionUsage`'s own body becomes
inline `subActions:`/`controlNodes:` data only, never a separate element.

## Rationale

Same traceability-symmetry rationale as `REQ-TRS-SYSMLV2-018` for `ActionDef`/`Action`. The depth
ceiling is a real, load-bearing fact about the dependency this project pins, not a design choice:
`FirstMergeBody::Brace` (the AST type backing `fork`/`join`/`decide`/`merge` bodies) carries no
data at all — the parser parses and then discards those block contents before this crate ever sees
them. No mapping design on Syscribe's side can recover what isn't there.

## Scope

- `ActionDef`/`ActionUsage` do **not** share one body type (`ActionDefBodyElement`/
  `ActionUsageBodyElement` are distinct, structurally near-identical Rust enums) — two separate
  top-level dispatch walks, sharing every per-construct handler beneath them, since the inner
  structs (`WhileStmt`/`IfStmt`/`ForkStmt`/…) are the same regardless of which body enum wraps them.
  `IfStmt.then_body`/`.else_body`, `WhileStmt.body`, `LoopStmt.body`, and `ForLoop.body` are all
  typed `ActionDefBody` regardless of the enclosing context, so recursion always flows through one
  walker.
- `subActions:` is a recursive, `kind:`-tagged tree: `PerformAction` (a `perform`/nested
  `ActionUsage`, `typedBy:` the referenced type), `AssignmentAction` (an `assign` statement, display
  text only), `LoopAction` (`while`/bare `loop`/`for`, with `loopKind:`/`condition:`/`variable:`/
  `sequence:` as applicable and a real, recursively-built `body:`), `IfAction` (`condition:`,
  `then:`, optional `else:`, each a real, recursively-built list), `TerminateAction`. Names for
  constructs the grammar itself gives none (`if`/`while`/`loop`/`for` carry no name field at all)
  are synthesized deterministically (`if_1`, `while_1`, …) — a Syscribe-owned naming convention.
- `controlNodes:` is flat and separate from `subActions:` — `ForkNode`/`JoinNode`/`DecisionNode`/
  `MergeNode` markers (`{name, kind}` only), bubbled up to the owning `ActionDef`/`Action`'s own
  top-level list regardless of how deeply the construct is nested inside `if`/`while`/`loop`
  bodies — matching the existing hand-authored convention (flat, not nested per branch).
- `successionConnections:` (the pre-existing `RawFrontmatter` field, already scanned generically by
  `W007`) carries `{after, before}` edges lifted from `first ... then ...`/bare `then ...`
  successions, tracking the most-recently-converted node as the implicit "after" side for the bare
  `then` shorthand.
- A nested `PartUsage`/`ItemUsage` inside an action body is unaffected — still becomes a real,
  separate element via the existing `convert_part_usage`/`convert_item_usage`, unchanged, since
  these are structural, not behavioral, constructs.
- Out of scope: a nested `ActionUsage`'s own body content is not recursed into when it appears as a
  `subActions:` reference (matches the hand-authored convention, which references sibling
  `ActionDef`s by `typedBy:` rather than inlining); `Bind`/`FlowUsage`/`AssertConstraint`/
  `OccurrenceUsage`/`DefaultReferenceUsage` and a `StateUsage` nested directly inside an action body
  stay unmapped.

**Acceptance criteria:** a package-wrapped `action def`/`action` synthesizes a real `ActionDef`/
`Action` element whose `subActions:`/`controlNodes:`/`successionConnections:` match the documented
convention; a `fork`/`join` block's `controlNodes:` entry carries only `name`/`kind`, confirmed to
carry no recoverable body content; `W080` (Sequence-diagram subject/edge completeness) sees a
SysMLv2-synthesized `ActionDef`'s real `subActions:` the same as a hand-authored one.
