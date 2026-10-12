---
type: PartDef
name: Sys
features:
  - {name: tx, typedBy: TxP}
  - {name: rx, typedBy: RxP}
connections:
  - {from: tx.out, to: rx.inp}
---
The system.
