---
type: Requirement
id: REQ-TRS-SYSMLV2-009
name: "SysML v2 doc /* ... */ comments lift into the synthesized element's doc body"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A `part def`/`part`/`interface def`/`interface` (usage)/`port def`/`port`/`connection def`/
`attribute def`/`attribute`/`item def`/`item` shall be able to declare one or more `doc /* ... */`
members. Syscribe shall lift their text, concatenated in source order, into the synthesized
element's `doc` field — the same field a hand-authored `.md` file's body populates below its
`---` closer — exactly as it would for a hand-authored element carrying the same text.

## Rationale

`RawElement.doc` already exists and is already read everywhere a hand-authored element's body is
read (`W600`/`W601`-style empty-doc checks, `show`, the web UI detail panel). Today every
SysMLv2-sourced element gets `doc: ""` unconditionally, regardless of what the `.sysml` source
actually says — `sysml-v2-parser` already parses `doc /* ... */` into a real, structurally-typed
`DocComment { identification, locale, text }` node, reachable as a `Doc(Node<DocComment>)` variant
in every relevant body-element enum; the text is fully available and simply discarded. This is
missing mapper coverage, not a parser limitation — the exact same posture `REQ-TRS-SYSMLV2-005`/
`-008` already established for `@SyscribeFeature`/`@Syscribe*` annotation lifting.

## Scope

- Covers the element kinds whose own body-element enum carries a `Doc` variant **and** which are
  already synthesized into a first-class element by `REQ-TRS-SYSMLV2-007`'s fixed set: `PartDef`,
  `Part`, `InterfaceDef`, `Interface`, `PortDef`, `Port`, `ConnectionDef`, `AttributeDef`,
  `Attribute`, `Item`, `ItemDef` (including a `variant part`/`variant attribute`/`variant
  port`/`variant item` usage, which share the same body shapes as their non-variant counterparts).
  `Item`'s own body is an `AttributeBody` — the same shared shape `AttributeDef`/`AttributeUsage`/
  `ItemDef` already use; `Interface`'s own `body_elements` is an `InterfaceUsageBodyElement`
  list, distinct from `InterfaceDef`'s `InterfaceDefBodyElement` — both confirmed against the
  parser's own struct definitions after two review rounds each caught one of these as incorrectly
  believed out of reach (an earlier draft claimed `ItemUsage` carries no body at all; a still
  earlier version of the implementation omitted `InterfaceUsageBodyElement::Doc` entirely,
  unnoticed until reviewed).
- Does **not** extend to `Package` or `Requirement`/`RequirementDef`/`RequirementUsage`, even
  though both are members of `REQ-TRS-SYSMLV2-007`'s fixed set and both have their own
  `Doc`-carrying body-element enum (`PackageBodyElement`, `RequirementDefBodyElement`) — a
  deliberate, matching-issue-scope descope, not an oversight; extending to either is candidate
  follow-on if a concrete need arises.
- Does **not** extend to constructs outside `REQ-TRS-SYSMLV2-007`'s mapped set (`calc`, `case`,
  `analysis`/`verification def`, …) — those synthesize no element at all, so there is nothing to
  attach lifted text to. Consistent with the module's existing parse-broad/map-narrow posture
  (`ADR-SYS-SYSMLV2-001` sub-decision 3), not a new exception to it. `state def`/`action def`
  moved into the mapped set as of `REQ-TRS-SYSMLV2-018`/`-019` and get their own doc lift there
  (`state_def_doc`/`action_def_doc`/`action_usage_doc`), reusing this requirement's `collect_doc`
  machinery unchanged.
- Multiple `doc` blocks on the same element **shall** concatenate in source order (joined by a
  blank line) rather than only the first or last one winning — the grammar permits several, and
  there is no reason to silently drop any of them.
- `identification`/`locale` on `DocComment` are not surfaced anywhere else in the frontmatter
  schema and are ignored by this requirement — only `text` matters.
- Text is carried verbatim — no Markdown rendering, reformatting, or reflow, exactly like a native
  element's body is carried verbatim — except that each individual `doc` block's own text is
  trimmed of the incidental whitespace directly adjacent to `/*`/`*/` (`sysml-v2-parser` includes
  it verbatim, e.g. `doc /* x */` parses to `" x "` not `"x"`; that padding is delimiter noise, not
  authored content). Internal formatting/newlines within a single block are left untouched. A
  `doc` block whose entire text is whitespace (`doc /* */`) contributes nothing to the join, the
  same as if it weren't written at all.
- `W600`/`W601`-style empty-doc-body warnings apply unchanged: a SysMLv2-sourced element with a
  non-empty lifted `doc` clears them exactly like a hand-authored one would; an element with no
  `doc` member still gets `doc: ""` and still trips them — no regression.

**Acceptance criteria:** a `part def` with `doc /* Explanation. */` gets `doc: "Explanation."` on
the synthesized element and no longer trips `W600`; a `part def` with two `doc` blocks gets both
texts concatenated in source order; a `part def`/`part`/etc. with no `doc` member is ingested
exactly as it is today (`doc: ""`, `W600` still fires) — no regression.
