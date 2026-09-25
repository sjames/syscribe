---
type: StateDef
name: SM
status: approved
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - target: Running
  - name: Running
    transitions:
      - name: halt
        accept: StopCommand
  - name: Stopped
    isFinal: true
transitions:
  - name: resume
    target: Running
    accept: ResumeCommand
  - source: Running
    target: Stopped
---

The top-level `resume` transition has no `source:` (W929, missing source); the nested
`halt` transition has no `target:` (W929, missing target).
