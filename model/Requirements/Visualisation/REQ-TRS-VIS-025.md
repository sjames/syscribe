---
type: Requirement
id: REQ-TRS-VIS-025
name: "A hand-listed Sequence diagram without pins is placed by the generator's sequence placement, never piled at the origin, and a fixed-algorithm layout is never run on unpinned nodes"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-021]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - sequence
---

A manifest-sourced `Sequence` diagram in which no shape carries a `layout:` pin shall be placed
by the same rules as a derived one (`REQ-TRS-VIS-021`) when its IR is built, so every renderer
draws it identically and none runs ELK on it:

- lifelines and actors (root shapes of those kinds) in declaration order, left to right at the
  fixed pitch, headers at the top;
- every edge whose two ends resolve — directly or through a parent chain — to a lifeline or actor
  column, in declaration order, as one row each, top to bottom, with horizontal waypoints from
  stem to stem (a self message as the small loop);
- each `activation` shape as a bar on its parent lifeline spanning the rows that touch that
  lifeline; a lifeline with several activations splits those rows between them in declaration
  order;
- each `fragment` shape as a box around the rows of the messages whose `ref` equals the
  fragment's `ref` or lies under it (`ref::…`), its label tab given its own vertical space, a
  fragment containing another drawn wider than it; a fragment matching no message is a small
  empty box under the headers;
- any other root shape (a note, a label) pinned in a row below the diagram, and any other child
  pinned at its parent's origin, so the whole IR is pinned.

A diagram that carries any pin keeps its author's pins and is not re-placed. Independently, the
layout of a graph that is not fully pinned shall never use ELK's `fixed` algorithm (which gives
every unplaced node the origin and rejects edges without sections): in the browser's layout
options and in `vis::layout`, `fixed` is used only when every node is pinned, otherwise the
layered algorithm.

## Rationale

`Diagrams::MissionExecutionSeq`, the demo's hand-listed sequence diagram, drew as a pile at the
origin in the browser and could not be exported at all (`ELK layout failed: The edge needs to have
exactly one edge section. Found: 0`), because only the derived generator knew how to place a
sequence diagram.

## Scope

- Message order is the order of the `edges:` map; the generator cannot reorder what the author
  listed. Fragment spans are inferred from `ref` and are a heuristic, documented as such.
