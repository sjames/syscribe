---
id: REQ-TRS-SYSMLV2-013
type: Requirement
name: A connect endpoint's dotted chain shall resolve to a direct nested feature of its head when one is actually declared, and otherwise to keep the full dotted path
status: verified
reqDomain: software
verificationMethod: test
---

A `connect` endpoint's two-segment dotted chain (`a.fooProvider`) **shall** be qualified to
`<owning qname>::<head>::<tail>` when the head is itself a `part` usage declared in the same body,
and that usage's own body declares a direct child named by the tail. Otherwise (GH #206) the whole dotted chain **shall** still map to a `::` path under the owner
(`<owner>::a::p1`), resolved by validation through the head's `typedBy:`/`supertype:` chain. A
three-or-more-segment chain **shall** map to its full `::` path likewise.

**Source:** `REQ-TRS-SYSMLV2-013` (product model).

**Acceptance criteria:** `part def Top { part a : A { interface fooProvider : IFoo; } part b : B {
interface fooClient : IFoo; } connection link1 : Link connect a.fooProvider to b.fooClient; }`
produces the edge `Top::a::fooProvider -> Top::b::fooClient`, resolvable via `connectivity`; the
same clause with neither `a` nor `b` redeclaring the referenced feature produces `Top::a::p1 ->
Top::b::p1`; a bare, undotted endpoint is unaffected; a three-segment chain keeps its full path.
