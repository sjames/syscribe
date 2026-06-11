---
id: REQ-TRS-MG-012
type: Requirement
title: trade-study shall treat an ambiguous parameterBindings variable match as unevaluable
status: draft
reqDomain: software
verificationMethod: test
---

The `trade-study` command ([[REQ-TRS-MG-007]]) resolves each MoE expression variable against a
`Configuration`'s `parameterBindings` by **exact key first**, then — as a convenience — by the
**final `.`/`::`-segment** of a binding key. When two or more bindings share that final segment
(e.g. `SubsysA.speed` and `SubsysB.speed`) and the expression uses the bare token (`speed`), the
match is **ambiguous**: the previous behaviour silently returned the first binding in iteration
order, which can yield a silently-wrong score. The tool **shall** instead treat an ambiguous
bare-token match as **unevaluable**.

### Behaviour

- An **exact** `parameterBindings` key match always wins and resolves the variable.
- Failing an exact match, if **exactly one** binding's final segment equals the token, the
  variable resolves to that binding's value (unchanged convenience behaviour).
- Failing an exact match, if **more than one** binding's final segment equals the token, the
  variable is **unresolved** — the MoE cell is reported `n/a` and excluded from that
  configuration's weighted-total normalisation (the same treatment as any other unevaluable
  cell, [[REQ-TRS-MG-007]]).

**Source:** robustness defect identified in the friction review of the `model_mg/` build — the
`trade-study` variable resolver could silently pick one of several colliding final-segment
bindings. Hardens [[REQ-TRS-MG-007]].

**Acceptance criteria:**

- A `Configuration` with bindings `SubsysA.speed` and `SubsysB.speed` and an MoE whose
  `expression` uses the bare token `speed` reports that cell as `n/a` (no guessed value).
- Adding an **exact** `speed` binding to the same configuration makes the cell resolve to that
  exact value despite the colliding segment keys.
- A configuration with a **single** `X.speed` binding still resolves the bare `speed` token (the
  convenience path is preserved).
