---
type: TradeStudy
id: TRD-SC-001
name: Non-numeric score
status: draft
criteria:
  - {name: c1, weight: 1.0, direction: maximize}
alternatives:
  - {name: a1}
scores:
  - {alternative: a1, criterion: c1, score: "abc"}
---
Score `abc` is not a number — E877.
