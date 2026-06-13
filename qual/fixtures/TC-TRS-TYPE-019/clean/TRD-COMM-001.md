---
type: TradeStudy
id: TRD-COMM-001
name: Communication Bus Trade
status: complete
objective: REQ-COMM-001
decision: ADR-COMM-001
criteria:
  - name: latency
    weight: 0.6
    direction: minimize
    unit: ms
  - name: cost
    weight: 0.4
    direction: minimize
    unit: USD
alternatives:
  - name: OptionA
  - name: OptionB
scores:
  - { alternative: OptionA, criterion: latency, score: 10 }
  - { alternative: OptionA, criterion: cost,    score: 5 }
  - { alternative: OptionB, criterion: latency, score: 20 }
  - { alternative: OptionB, criterion: cost,    score: 3 }
---

A complete trade study; OptionA wins on latency, OptionB on cost.
