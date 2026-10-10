---
type: Requirement
id: REQ-TRS-BDDX-001
name: "A derived BDD follows composition across packages and accepts qualified include entries"
status: draft
reqDomain: software
reqClass: system
tags:
  - vis
---

A derived BDD shall show the blocks a subject composes even when they live in other packages (GH #243).

## Behavior

- Composition targets outside the subject's own members are pulled in as **external blocks** (a `Block` node whose mark status is `external`), with the composition edge drawn to them. This applies to a `Package` subject (parts of its members typed by a definition elsewhere) and to a `PartDef`/`ItemDef` subject (its composed blocks).
- The new `depth:` key on the `Diagram` limits how many composition levels are followed from the subject's own blocks: default `1`, `0` restores the previous behaviour (only direct members), `2` also pulls in what the external blocks compose, and so on. Cycles terminate.
- A package subject's `include:` accepts a name qualified relative to the subject (`Hardware::Box`) or a full qualified name or id, and the entry then counts as matched (no `W417`); such an entry adds that definition as a block even when it is not a direct member.
- Edges are still drawn only between nodes on the diagram; `exclude:` still removes a block (an excluded external block is not pulled in).
