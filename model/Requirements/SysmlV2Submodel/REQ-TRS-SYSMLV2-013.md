---
type: Requirement
id: REQ-TRS-SYSMLV2-013
name: "A connect endpoint's dotted chain shall resolve to a direct nested feature of its head when one is actually declared, falling back to the head-only edge otherwise"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - connectivity
---

A `connect` endpoint's two-segment dotted chain (`a.fooProvider`) shall be qualified to
`<owning qname>::<head>::<tail>` — resolving to the nested feature, not just the owning part —
when the chain's head (`a`) is itself a `part` usage declared in the same body, and that usage's
own body in turn declares a direct child (`port`/`attribute`/`interface`/nested `part` usage)
named by the tail (`fooProvider`). When no such match exists, `REQ-TRS-SYSMLV2-010`'s existing
head-only qualification is unchanged — this is a strict widening, never a new failure mode.

## Rationale

`REQ-TRS-SYSMLV2-010` deliberately truncated every connect endpoint to its head segment, because a
trailing segment is overwhelmingly a feature *inherited* from the head's type rather than
redeclared on the usage, and this module does no inheritance resolution — chasing it would have
required exactly that. But when a `.sysml` author explicitly redeclares the feature on the usage
itself (`part a : Ecu { interface fooProvider : IFoo; }`), the trailing segment genuinely does
correspond to a real, already-parsed sibling node — truncating it there is needlessly lossy for a
case this module can resolve locally, with no inheritance reasoning at all.

## Scope

- Resolution is purely local to the connect clause's own enclosing body — no global/cross-file
  resolver access, no element-graph lookups. The owning part's already-parsed body-element list
  (the same slice `REQ-TRS-SYSMLV2-010`'s lift already has in hand) is searched for a `part` usage
  named by the head; if found, *that* usage's own already-parsed body is searched for a direct
  child named by the tail. Both steps are pure AST inspection, at ingest time, before any
  `RawElement` synthesis for the referenced children has even happened.
- Only a genuinely **two-segment** chain (`head.tail`, no further `.`) is eligible for this
  resolution. A chain with more than two segments (`a.b.c`) falls back to head-only qualification
  unconditionally — extending the local-lookahead search to multiple levels is not attempted.
- The tail may match a `port`/`attribute`/`interface` usage, or a nested `part` usage, declared
  directly in the head's own body. An `item` usage is not matched — `PartUsageBodyElement` (the
  body-element enum of a `part` *usage*, as opposed to a `part def`) carries no `ItemUsage`
  variant at all in this grammar, so a `part` usage cannot declare a nested `item` usage to begin
  with; nothing is being excluded here that could otherwise match.
- The head must itself be a `part` usage (`PartDefBodyElement::PartUsage`/`PartUsageBodyElement::PartUsage`)
  declared in the *same* enclosing body as the `connection` usage. A head that resolves to
  something else (a `port`/`attribute` sibling, or a `part def` rather than a usage) falls back to
  head-only qualification — the same outcome as today.
- A trailing segment that doesn't match anything in the head's own body (the common,
  inherited-feature case `REQ-TRS-SYSMLV2-010` already handles) falls back to head-only
  qualification exactly as before — not an error, not a warning, no new diagnostic.
- Still no inheritance resolution of any kind — this requirement only reaches a feature that is
  *explicitly* redeclared on the usage, never one inherited from a type.

**Acceptance criteria:** `part def Top { part a : A; part b : B; connection link1 : Link connect
a.fooProvider to b.fooClient; }` where `a`'s own body declares `interface fooProvider : IFoo;` and
`b`'s own body declares `interface fooClient : IFoo;` produces the edge
`Top::a::fooProvider -> Top::b::fooClient`; the same connect clause where neither `a` nor `b`
redeclares the referenced feature produces `Top::a -> Top::b` exactly as before
(`REQ-TRS-SYSMLV2-010`'s existing behavior, unchanged); a bare, undotted endpoint (`connect a to
b;`) is unaffected; a three-or-more-segment chain falls back to head-only qualification.
