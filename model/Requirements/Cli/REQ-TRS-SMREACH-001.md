---
type: Requirement
id: REQ-TRS-SMREACH-001
name: "W070 and W071 follow transitions declared on a state's ancestors and inside its descendants"
status: implemented
reqDomain: software
reqClass: system
tags:
  - state-machine
  - validation
---

`W070` (dead state) and `W071` (trap state) shall judge a state against every transition of the machine, not only those written at its own level (GH #515).

- **Entered.** A state is not dead when any transition anywhere in the machine targets it: one declared on an ancestor (a composite whose transition targets one of its children), one declared inside a descendant of a sibling (a nested state leaving outwards), or a top-level `transitions:` entry. A target written as a path (`Live.Parked`, `Live::Parked`) names the state by its last segment.
- **Left.** A state is not a trap when it, any ancestor, or any descendant is the source of a transition, because an ancestor's transition leaves every state inside it and a descendant's transition may leave the composite.
- **Unchanged.** `isInitial` still exempts a state from `W070`, `isFinal` from `W071`, and a state that no transition enters (or leaves) at any level is still reported.
