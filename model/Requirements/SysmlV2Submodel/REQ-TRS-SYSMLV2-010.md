---
type: Requirement
id: REQ-TRS-SYSMLV2-010
name: "A named connection usage's connect a to b (, c)* endpoints lift into resolvable connectivity/n2 graph edges on the owning part"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - connectivity
---

A `part def`/`part`'s named `connection name : Type connect a to b (, c)*;` usage member shall
have its `connect_from`/`connect_to`/`connect_extra_ends` endpoints lifted onto the **owning**
`part def`/`part`'s `connections:` field — the same field a hand-authored `.md` file's
`connections: [{from, to}]`/`connections: [{ends: [...]}]` populates — so `connectivity` (and
unscoped `n2`; see the Rationale's disclosed limitation) show real, resolvable off-diagonal wiring
for a `sysmlSubmodel: true` subtree, not just missing data.

## Rationale

Real SysML v2 architecture is fundamentally about *interfaces between* components, not just the
components themselves. A `sysmlSubmodel: true` package can already carry every structural element
(`REQ-TRS-SYSMLV2-007`) but, until this requirement, none of the wiring between them —
`connectivity`/`n2` show no wiring at all regardless of how many `connect` clauses the source
actually contains, because `ConnectionUsageMember.connect_from`/`connect_to`/`connect_extra_ends`
are fully parsed by `sysml-v2-parser` and simply never read.

**Disclosed limitation, not fixed by this requirement (resolved by `REQ-TRS-SYSMLV2-011`):**
`n2 <qname>` (**scoped** to a specific element) builds its row/column axis from the scope
element's own `features:` list (`crates/syscribe/src/n2.rs::subpart_axis`) exclusively — a
SysMLv2-synthesized `part def`/`part` never populates `features:` (its subparts are separate
synthesized children, `REQ-TRS-SYSMLV2-002`), so scoped `n2` on any SysMLv2 subtree reports `(no
parts in scope)` regardless of this requirement. Only **unscoped** `n2` (the bare `n2` command,
whose axis is every `PartDef`/`Part` in the whole model, not `features:`-derived) and
`connectivity` benefit from this lift as originally shipped. This was a pre-existing `n2.rs`
limitation out of this requirement's own scope — disclosed here rather than left to be discovered
as a surprise — and was closed shortly after by `REQ-TRS-SYSMLV2-011`, which widens
`subpart_axis` to also include SysMLv2-synthesized children by qname containment.

## Scope

- Covers the named `connection name : Type connect ...;` usage form (`ConnectionUsageMember`,
  dispatched as `PartDefBodyElement::Connection`/`PartUsageBodyElement::Connection` — note this is
  a different, distinct AST variant from `PartDefBodyElement::Connect`/`PartUsageBodyElement::Connect`,
  which is the truly anonymous binary-connector form and stays unmapped, matching this module's
  existing precedent for other anonymous forms with no name to synthesize an identity against).
- A named connection usage with **no** `connect` clause (`connection c : SomeConnDef;`, the form
  already supported before this requirement) is unaffected — no `connections:` entry is
  contributed for it, no regression.
- The synthesized standalone `Connection` element itself (`REQ-TRS-SYSMLV2-007`'s existing
  structural-browsing mapping) is unchanged by this requirement — the lifted endpoints attach to
  the **owning** `part def`/`part`, not to the nested `Connection` element.
- Covers a `connection` usage nested inside a `variant part` usage the same way it covers an
  ordinary `part def`/`part` — `variant part quadConfig : T { connection c : Def connect a to b;
  }` lifts onto `quadConfig`'s own `connections:` exactly like a non-variant part usage would.
- **Endpoint qualification (the one deliberate divergence from a literal transcription of the
  `.sysml` source text — see the `ADR-SYS-SYSMLV2-001` addendum for the full, two-round
  investigation):** each endpoint's dotted feature-chain text (e.g. `a.p1`) is rewritten into a
  qualified qname before being written into `connections:` — the owning part's own qname followed
  by `::` and only the chain's **head** segment (`a.p1` under `Holder` becomes `Holder::a`, not
  `Holder::a::p1`) — rather than carried verbatim or fully re-joined with `::`. A literal,
  unqualified chain never resolves to a graph edge in this codebase's existing connection-edge
  resolver; a naive full-chain-to-`::` conversion mostly doesn't either, since a trailing segment
  like `p1` is overwhelmingly a port *inherited* from the head's type rather than redeclared on
  the usage, and this module does no inheritance resolution (both confirmed empirically before
  settling on head-only). Head-only qualification matches this same connection graph's own
  existing precedent for `features:`-declared endpoints exactly (head resolved, rest of the chain
  discarded) — reached via the resolver's exact-qname match against a real synthesized instance
  instead, with no resolver changes.
- A trailing segment past the head is therefore always discarded, not just when it happens not to
  resolve — this is a deliberate granularity choice (instance-level, not port-level), not a
  best-effort resolution attempt that sometimes succeeds. Not itself a validation finding.
- The n-ary `connect (a, b, c)` form (`connect_extra_ends` non-empty) lifts to the same `ends:
  [{binds: ...}, ...]` shape a hand-authored n-ary `connections:` entry already uses (no
  role labels — SysML v2's own `connect` grammar carries no per-end role name to preserve).
- The connection usage's own `type_name` (its `Type` in `connection name : Type connect ...;`)
  lifts into the entry's `typedBy:`, matching the hand-authored convention.
- Still read-only, one-way ingestion — no writer/round-trip.

**Acceptance criteria:** `part def Foo { part a : Ecu; part b : Ecu; connection c : SomeConnDef
connect a.p1 to b.p1; }` produces a `connections:` entry on `Foo` resolving to a real
`connectivity` edge between `Foo::a` and `Foo::b`; the n-ary `connect (a, b, c)` form produces an
`ends:`-shaped entry with all ends present, each verified resolvable via `connectivity` (`n2`'s
own edge-collection reads only the first two `ends:` entries for any n-ary connection, native or
SysMLv2-lifted alike — a pre-existing, unrelated `n2.rs::collect_edges` limitation, not this
requirement's regression, and not its job to fix — so `connectivity`, which correctly builds a
full star over every end, is the acceptance-criteria tool of record for the n-ary case); a named
connection usage with no `connect` clause contributes no entry (no regression); the anonymous (no
`connection name :` prefix) form remains unmapped; a `connection` usage nested inside a `variant
part` usage lifts onto that usage's own `connections:` the same way.
