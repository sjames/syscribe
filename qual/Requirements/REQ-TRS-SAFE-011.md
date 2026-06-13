---
id: REQ-TRS-SAFE-011
type: Requirement
name: "safety-case shall suppress implicit fold-in for goals that have explicit supporting Arguments"
status: draft
reqDomain: software
verificationMethod: test
---

When a `SafetyGoal` has at least one explicit `Argument` in its `support:` list, the
`safety-case` command **shall** suppress the implicit fold-in of `derivedFromSafetyGoal`
`Requirement` nodes (and their verifying `TestCase` children) directly under the goal.

**Rationale**: when an explicit GSN argument layer exists, the implicit chain
(`SafetyGoal → Requirement → TestCase`) is already expressed inside the argument's
`evidence:` references. Printing both creates visually confusing duplicates.

The suppression **shall** apply per-goal: a goal with no supporting `Argument` still
receives the implicit fold-in as before.

The `--no-implicit` flag, when passed to `safety-case`, **shall** additionally suppress
the implicit fold-in for all goals regardless of whether they have explicit Arguments.

**Acceptance criteria:**

- A goal with supporting `Argument`(s) does **not** show the implicit
  `[evidence:Requirement]` section directly under it.
- A goal with **no** supporting `Arguments` **does** show the implicit fold-in
  (existing behaviour preserved).
- `--no-implicit` suppresses the fold-in for all goals.
- The JSON output also omits `"requirements"` entries when the goal has explicit
  Arguments (or `--no-implicit` is passed).
