---
id: REQ-TRS-SM-006
type: Requirement
name: Tool shall validate composite (hierarchical) state machines recursively
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise **composite (hierarchical) states** per SysMLv2 §7.18
(`temp/sysml2_spec.pdf`): a substate is composite when it carries `typedBy:` (a reference to
another `StateDef`) or an inline `subStates:` list (a nested state machine).

The §22.1 completeness checks **shall** apply **recursively** over the state hierarchy:

- At each level (the top `StateDef` and every nested region), the level's direct substates
  **shall** be checked by the flat rules `W070`–`W074`, with a composite substate treated as
  a **single node** in that level's transition graph (its inner detail is checked
  separately).
- For every substate that declares an **inline `subStates:`** list, the tool **shall**
  recurse and check that nested region's own completeness (its initial cardinality, dead and
  trap states), with findings naming the enclosing region.
- A substate that is composite **by reference** (`typedBy:` a `StateDef`) **shall not** be
  recursed into here — the referenced `StateDef` is a first-class element validated in its
  own right; at the enclosing level it is just a node.

The flat single-region behaviour of [[REQ-TRS-SM-003]] is the depth-1, non-composite special
case of this recursion; parallel regions ([[REQ-TRS-SM-004]]) are checked per region by the
same recursion. A composite substate **shall no longer** suppress the dead/trap checks of its
enclosing level.

**Source:** SysMLv2 §7.18 (composite states / state decomposition). Generalises the flat and
per-region checks to arbitrary nesting.

**Acceptance criteria:**

- A composite machine whose top level and every nested region are well-formed raises none of
  W070–W074.
- A nested region (inside an inline-`subStates:` substate) that has no initial state raises
  `W073` naming the enclosing region.
- A dead or trap substate at the **top** level of a composite machine is flagged even though
  the machine contains a composite substate (composite no longer suppresses the level).
- A `typedBy:` composite substate is treated as a node and is not recursed into.
