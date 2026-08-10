---
id: REQ-TRS-SYSMLV2-013
type: Requirement
name: A connect endpoint's dotted chain shall resolve to a direct nested feature of its head when one is actually declared, falling back to the head-only edge otherwise
status: draft
reqDomain: software
verificationMethod: test
---

A `connect` endpoint's two-segment dotted chain (`a.fooProvider`) **shall** be qualified to
`<owning qname>::<head>::<tail>` when the head is itself a `part` usage declared in the same body,
and that usage's own body declares a direct child named by the tail. Otherwise the existing
head-only qualification **shall** apply unchanged. A three-or-more-segment chain **shall** always
fall back to head-only.

**Source:** `REQ-TRS-SYSMLV2-013` (product model).

**Acceptance criteria:** `part def Top { part a : A { interface fooProvider : IFoo; } part b : B {
interface fooClient : IFoo; } connection link1 : Link connect a.fooProvider to b.fooClient; }`
produces the edge `Top::a::fooProvider -> Top::b::fooClient`, resolvable via `connectivity`; the
same clause with neither `a` nor `b` redeclaring the referenced feature produces `Top::a ->
Top::b` exactly as before; a bare, undotted endpoint is unaffected; a three-segment chain falls
back to head-only.
