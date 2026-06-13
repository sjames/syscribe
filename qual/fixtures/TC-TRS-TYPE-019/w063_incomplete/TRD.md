---
type: TradeStudy
id: TRD-IN-001
name: Incomplete matrix
status: review
criteria:
  - {name: c1, weight: 0.5, direction: maximize}
  - {name: c2, weight: 0.5, direction: maximize}
alternatives:
  - {name: a1}
  - {name: a2}
scores:
  - {alternative: a1, criterion: c1, score: 5}
  - {alternative: a1, criterion: c2, score: 3}
  - {alternative: a2, criterion: c1, score: 4}
---
Missing a2×c2 score — W063.
