---
id: REQ-TRS-SYSMLV2-010
type: Requirement
name: A named connection usage's connect endpoints shall lift into resolvable connectivity/n2 graph edges on the owning part
status: draft
reqDomain: software
verificationMethod: test
---

A `part def`/`part`'s named `connection name : Type connect a to b (, c)*;` usage member **shall**
have its endpoints lifted onto the **owning** `part def`/`part`'s `connections:` field — the same
field a hand-authored `.md` file's `connections:` populates — such that `n2`/`connectivity` show
real, resolvable off-diagonal wiring, not just copied text. Each endpoint's dotted chain **shall**
be qualified to `<owning qname>::<head segment only>` before being written, since neither a
literal nor a fully-`::`-qualified chain resolves to a real graph edge in this codebase's existing
connection-edge resolver (confirmed by investigation, not assumed).

A named connection usage with **no** `connect` clause **shall** be ingested exactly as it is
today — no entry contributed, no regression. The anonymous (no `connection name :` prefix) form
**shall** remain unmapped.

**Source:** `REQ-TRS-SYSMLV2-010` (product model).

**Acceptance criteria:** `part def Foo { part a : Ecu; part b : Ecu; connection c : SomeConnDef
connect a.p1 to b.p1; }` produces a `connections:` entry on `Foo` resolving to a real graph edge
between `Foo::a` and `Foo::b` (verified via `connectivity`, not just the presence of the
`connections:` field); the n-ary `connect (a, b, c)` form produces an `ends:`-shaped entry with
every end resolving to a real edge; a connection usage with no `connect` clause contributes no
entry.
