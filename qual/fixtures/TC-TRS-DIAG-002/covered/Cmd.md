---
type: ActionDef
name: Cmd
subActions:
  - name: sendCmd
    kind: SendAction
    payload: Items::Signal
    via: out
---

Command action whose `sendCmd` SendAction is covered by the sequence diagram.
