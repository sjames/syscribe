---
type: StateDef
name: SM
subStates:
  - name: Init
    isInitial: true
    transitions:
      - target: A
        accept: Items::Cmd
      - target: B
        accept: Items::Cmd
  - name: A
    isFinal: true
  - name: B
    isFinal: true
---

`Init` has two unguarded transitions accepting the same payload `Items::Cmd` — W072. `A`/`B`
are final, so no trap; both have incoming, so no dead state.
