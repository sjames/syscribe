---
id: REQ-TRS-SAFE-012
type: Requirement
name: Tool shall validate ASIL D / SIL 4 decomposition pair completeness (E865, W860)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** extend the integrity-level propagation rules (§12.7, Rule R-007) with a
structural check on **ASIL D / SIL 4 decomposition** claims, per ISO 26262-9 §5 /
IEC 61508-2 §7.4.9. A decomposition exists when a `Requirement` at `asilLevel: D` (or
`silLevel: 4`) has `derivedChildren` that **all** carry a strictly lower integrity level
("uniformly lower"). Over such a parent:

| Code | Condition |
|---|---|
| `E865` | Two decomposition sibling requirements (the uniformly-lower children) name the **same** architecture element in their `satisfies:` lists — the decomposed channels are not architecturally independent. |
| `W860` | The decomposition has **fewer than two** children (a single uniformly-lower child) — a one-channel ASIL D / SIL 4 decomposition is structurally incomplete. |

> **Code note:** the format spec drafted these as `E860`/`W860`, but `E860` is already taken
> (ConfirmationMeasure/CybersecurityGoal type check, release 0.26.7). The decomposition
> error is therefore assigned **`E865`** (next free in the safety block); `W860` is
> unchanged. The spec §22.3 is corrected to match.

`E865` is an error; `W860` is a warning, draft-suppressed and gateable with `--deny W860`.
The check applies only when the parent is `asilLevel: D` or `silLevel: 4` and **every**
integrity-bearing child is strictly lower (a genuine decomposition); a parent with a
same-level child is not a decomposition and raises neither code.

### New `Requirement` field — `decompositionKind`

The tool **shall** recognise an optional `decompositionKind:` field on `Requirement` with
enum values `independent` | `redundant` | `diverse` (informational). It **shall** be:

- surfaced in the `safety-case` report output for the requirement;
- emitted as a commented hint in the `template Requirement` skeleton (for ASIL D / SIL 4
  decomposition).

**Source:** §22.3 (ASIL/SIL Decomposition Pair Completeness, extends §12.7), GH #69.
Builds on the integrity-propagation rules `E841`–`E843`/`W808` and the `derivedChildren`
reverse index.

**Acceptance criteria:**

- `E865` fires when two lower-level children of an ASIL D (or SIL 4) parent share a
  `satisfies:` target; it does not fire when the siblings satisfy distinct elements.
- `W860` fires when an ASIL D / SIL 4 parent has exactly one uniformly-lower child.
- Neither fires when the parent is not ASIL D / SIL 4, or when a child retains the parent's
  level.
- `decompositionKind:` appears in the `safety-case` report for a requirement that sets it.
- `template Requirement` includes a commented `decompositionKind:` line.
- The shipped models remain clean (no ASIL D / SIL 4 requirement in them is a parent).
