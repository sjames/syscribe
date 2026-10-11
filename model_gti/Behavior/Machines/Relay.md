---
type: StateDef
name: Relay
variables:
  - {name: closed, value: "0"}
  - {name: inhibit, value: "0"}
subStates:
  - name: Open
    isInitial: true
    transitions:
      - {target: Closed, accept: {payload: Close, via: cmd}, guard: "inhibit == 0", assign: "closed := 1"}
      - {internal: true, accept: {payload: Trip, via: trip}, assign: "inhibit := 1"}
      - {internal: true, accept: {payload: Rearm, via: trip}, assign: "inhibit := 0"}
  - name: Closed
    transitions:
      - {target: Open, accept: {payload: Shutdown, via: cmd}, assign: "closed := 0"}
      - {target: Open, accept: {payload: Trip, via: trip}, assign: "closed := 0; inhibit := 1"}
---

One of the two series grid relays. `closed` is the contact position; a welded contact is modelled by pinning `closed` at 1. The hardware trip also sets `inhibit`, which keeps the relay from closing until the trip has re-armed (relay B only: relay A has no hardware connection).
