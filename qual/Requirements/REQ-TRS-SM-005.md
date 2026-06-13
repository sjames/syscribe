---
id: REQ-TRS-SM-005
type: Requirement
name: Tool shall flag illegal cross-region transitions (W077) and parallel arity (W078)
status: draft
reqDomain: software
verificationMethod: test
---

For a parallel state machine ([[REQ-TRS-SM-004]]), the tool **shall** enforce two
structural rules from SysMLv2 §7.18 ("no transitions are allowed between the substates of a
parallel state"):

| Code | Condition |
|---|---|
| `W077` | A transition whose `source` and `target` resolve to substates belonging to **different** regions of the same parallel state. Region membership is taken from each region's direct substate names; a name appearing in more than one region is ambiguous and excluded from the check. |
| `W078` | An `isParallel: true` state that declares **fewer than two** regions (direct substates). A parallel state with one region is not orthogonal. |

Both **shall** be **draft-suppressed** and **gateable** with `--deny W077` / `--deny W078`,
consistent with the other state-machine warnings.

The transition scan for `W077` **shall** consider every edge of the machine — the parallel
parent's own top-level `transitions:` and each region's internal transitions — so a
cross-region edge declared at any level is caught.

**Source:** SysMLv2 §7.18 (parallel-state well-formedness); companion to the per-region
completeness of [[REQ-TRS-SM-004]].

**Acceptance criteria:**

- A top-level transition connecting a substate in one region to a substate in another raises
  `W077`.
- A transition that stays within a single region raises no `W077`.
- An `isParallel: true` state with one region raises `W078`; with two or more, none.
- Both are suppressed for `status: draft` and gate non-zero under `--deny W077` / `--deny W078`.
