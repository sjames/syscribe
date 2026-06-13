---
type: TradeStudy
id: TRD-EL-001
name: Unresolved alternative element
status: review
criteria:
  - {name: c1, weight: 1.0, direction: maximize}
alternatives:
  - {name: a1, element: Nonexistent::Part}
scores:
  - {alternative: a1, criterion: c1, score: 5}
---
Alternative `element` does not resolve — W064.
