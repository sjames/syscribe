---
id: REQ-TRS-VIS-025
type: Requirement
name: A hand-listed Sequence diagram without pins is placed by the generator's sequence placement, and a fixed layout is never run on unpinned nodes
status: verified
reqDomain: software
verificationMethod: test
---

A manifest `Sequence` diagram in which no shape carries a pin **shall** be placed when its IR is
built, so no renderer runs ELK on it: lifelines and actors in declaration order at the fixed
pitch; one row per edge whose ends reach a column (directly or via `parent:`) in declaration
order with horizontal waypoints (a self message loops); activation bars spanning the rows that
touch their lifeline, several on one lifeline sharing them in order; fragment boxes around the
messages whose `ref` equals or lies under the fragment's, an enclosing fragment wider; every
other shape pinned too. An unlabelled message **shall** be labelled by the last segment of its
`ref`, lifeline headers **shall** carry no banners, and a diagram with any pin **shall** keep the
author's pins. A graph that is not fully pinned **shall** never be given ELK's `fixed` algorithm.

**Source:** `REQ-TRS-VIS-025` (product model).

**Acceptance criteria:** (a) the whole IR is pinned with waypoints on every message and the
unplaced note below the diagram; (b) rows go down by at least a row pitch, messages run stem to
stem, a return runs from its source stem, a self message has four waypoints; (c) unlabelled
messages are labelled by their ref and an author's label wins; (d) fragments enclose their rows
and the outer is wider than the inner; (e) two activations on one lifeline do not overlap and
the second starts later; (f) a diagram with one pin is untouched; (g) it renders without ELK;
(h) the demo model's `Diagrams::MissionExecutionSeq` builds with no issue, is fully pinned and
exports (it failed with an ELK error before); (i) an unpinned sequence graph's layout options use
`layered`, a fully pinned one `fixed`.
