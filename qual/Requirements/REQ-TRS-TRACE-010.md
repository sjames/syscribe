---
id: REQ-TRS-TRACE-010
type: Requirement
name: Unsatisfied Safety-Mechanism Check
title: Tool shall flag a high-integrity requirement that is draft, unsatisfied, or active in no configuration (W306)
status: draft
reqDomain: software
verificationMethod: test
---

The most serious recurring audit finding is a requirement that is simultaneously **high-integrity** and **immature/unintegrated** — a safety mechanism that is not yet a real, claimed, baseline part of the system. The tool **shall** make this a first-class, gateable check.

## `W306`

The tool **shall** emit warning **`W306`** for a native `Requirement` whose integrity level meets the threshold — **`silLevel >= 4` OR `asilLevel == D`** (the highest tiers; the default threshold) — when **any** of these sub-conditions holds:

1. `status: draft` — the mechanism is not yet a committed requirement;
2. **unsatisfied** — no architecture element `satisfies:` it (its `satisfied_reqs` set is empty);
3. **active in no configuration** — when a feature model is present and ≥1 `Configuration` exists, the requirement's effective `appliesWhen:` evaluates false for **every** `Configuration` (it ships in no baseline; N/A across the whole coverage matrix).

The finding message **shall name which sub-condition(s)** triggered, so the author knows whether the gap is maturity, assignment, or baseline inclusion. `W306` is opt-in (it cannot fire below the integrity threshold), **gateable** via `validate --deny W306`, and promotable to a gating error through the severity-profile mechanism ([[REQ-TRS-VAL-003]]; full threshold/sub-condition configuration is delivered with the named-profiles work, GH #18).

**Source:** GH #17. Composes the maturity (`status`), assignment (`W300`, §12.3), and projection/coverage (matrix N/A) signals into one safety-readiness rule.

**Acceptance criteria:** a `silLevel: 4` (or `asilLevel: D`) requirement that is `draft`, or that no element satisfies, or that is active in no configuration, produces exactly one `W306` naming the triggering sub-condition(s); a high-integrity requirement that is non-draft, satisfied, and active in some configuration produces none; a requirement below the integrity threshold never produces `W306`; `validate --deny W306` exits non-zero when a `W306` is present.
