---
id: REQ-TRS-SAFE-011
type: Requirement
name: "safety-case shall fold in the implicit requirement chain for every goal and expose completeness"
status: draft
reqDomain: software
verificationMethod: test
---

The `safety-case` command **shall** fold the implicit `SafetyGoal ← Requirement
(derivedFromSafetyGoal) ← TestCase (verifies)` chain under every goal, including goals
that have explicit supporting `Argument`s (GH #217; this supersedes the earlier rule that
suppressed the chain whenever a goal had an `Argument`, which hid incomplete arguments).

**Rationale**: an explicit GSN argument layer may cite only some of the requirements
derived from the goal. Suppressing the rest made a partial argument look complete.

A derived requirement that an `Argument` already cites under the same goal **shall** be
shown once, under the `Argument`, and not repeated as implicit. Requirements **shall** be
expanded through their `derivedChildren` transitively down to the verifying `TestCase`s.

The `--no-implicit` flag **shall** suppress the implicit fold-in for all goals.

The command **shall** mark nodes that have no supporting evidence as undeveloped, print a
per-goal verdict and a completeness summary (text and `--json`), and **shall** exit
non-zero when a named goal id matches no `SafetyGoal`.

**Acceptance criteria:**

- A goal with supporting `Argument`(s) still shows a derived requirement the `Argument`
  does not cite, marked `(implicit)`; a requirement the `Argument` cites appears once.
- A goal with no supporting `Arguments` shows the implicit fold-in (existing behaviour).
- `--no-implicit` suppresses the fold-in for all goals.
- The JSON output lists the implicit requirements, per-goal `verdict`, per-node `status` and
  `undeveloped`, and a `completeness` object.
- `safety-case <unknown-id>` exits non-zero.
