---
id: REQ-TRS-VIS-008
type: Requirement
name: Create, delete, port-aware connect and move work on manifest and derived diagrams through the guarded-write engine
status: verified
reqDomain: software
verificationMethod: test
---

The editing gestures of `REQ-TRS-DE-004` **shall** work on every IR-backed diagram: *create*
makes the element, its manifest shape and a pin in one guarded write on a manifest diagram, and
the element alone under the subject on a derived one; *delete* removes the element and prunes its
shapes and edges from every manifest; *move* writes a pin on either.

The connect gesture **shall** be **port-aware**. Two ports **shall** be joined when their
directions are compatible (`out`↔`in`; an `inout` or undirected port with anything) and refused
when both have the same direction. A block **shall** stand in for its single compatible port: when
one or both ends are blocks, exactly one compatible (source port, target port) pair is accepted,
none is refused, and more than one is refused as ambiguous; a block with no ports is refused.
Every refusal **shall** be an explanatory toast with nothing written. An accepted gesture
**shall** add a `connections:` entry on the diagram's `subject:` whose `from`/`to` are dotted
feature chains relative to that owner (`battery.powerOut`), and on a manifest diagram also the
edge in the manifest, in one transaction; on a derived diagram no `diagram:` sync block **shall**
be sent. Every write **shall** go through `syscribe_model::mutate`'s guarded-write engine and
surface the `WriteResponse` delta; a refusal **shall** revert the optimistic change.

**Source:** `REQ-TRS-VIS-008` (product model).

**Verification:** the connect rules live in `crates/syscribe-server/frontend/src/connect-rules.ts`
as pure functions and are exercised by `TC-TRS-VIS-008`, a Node test
(`crates/syscribe-server/frontend/test/connect-rules.test.mjs`, run by `npm test`) driving
`resolveConnectEnds`, `compatible`, `portChain` and `isDerivedDiagram` over fixture schemas. The
guarded-write round trip of create/delete/move is covered by the existing
`REQ-TRS-DE-004`/`-005` cases.

**Acceptance criteria:** (a) port→port with `out`→`in` is accepted;
(b) `out`→`out` is refused naming both directions; (c) a block with one compatible port stands in
for it; (d) a block with two compatible ports is refused as ambiguous, listing the pairs; (e) a
block with no ports is refused; (f) `portChain` spells a nested port as `block.port` relative to the
subject and the subject's own port as its bare name; (g) `isDerivedDiagram` is true only when the
diagram has a subject and every root shape id is its ref's slug.
