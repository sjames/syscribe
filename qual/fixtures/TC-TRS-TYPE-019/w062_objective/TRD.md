---
type: TradeStudy
id: TRD-OB-001
name: Unresolved objective
status: review
objective: Nonexistent::Requirement
criteria:
  - {name: c1, weight: 1.0, direction: maximize}
alternatives:
  - {name: a1}
scores:
  - {alternative: a1, criterion: c1, score: 5}
---
`objective` does not resolve — W062.
