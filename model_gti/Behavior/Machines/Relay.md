---
type: StateDef
name: Relay
variables:
  - {name: closed, value: "0"}
subStates:
  - name: Open
    isInitial: true
    transitions:
      - {target: Closed, accept: {payload: Close, via: cmd}, assign: "closed := 1"}
  - name: Closed
    transitions:
      - {target: Open, accept: {payload: Shutdown, via: cmd}, assign: "closed := 0"}
      - {target: Open, accept: {payload: Trip, via: trip}, assign: "closed := 0"}
---

One of the two series grid relays. `closed` is the contact position; a welded contact is modelled by pinning `closed` at 1.
