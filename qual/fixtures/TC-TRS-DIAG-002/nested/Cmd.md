---
type: ActionDef
name: Cmd
subActions:
  - name: check
    kind: IfAction
    condition: "fault == true"
    then:
      - name: abort
        kind: SendAction
        payload: Items::Signal
        via: out
    else:
      - name: proceed
        kind: PerformAction
        typedBy: Cmd
---

Command action whose `abort` SendAction lives inside an IfAction then-branch and is not
covered by the sequence diagram — exercises the recursive walk.
