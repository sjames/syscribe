---
type: AttackTree
id: AT-ENG-001
name: Attack tree for TS-ENG-001 — replayed CAN torque request
status: approved
threatRef: TS-ENG-001
---

Two ways to get a forged torque request onto the powertrain CAN: sit at the OBD-II port with a
recorded frame (a path, so every step is needed), or take over the telematics unit and inject
frames remotely. The weakest link rule rolls the tree up to the feasibility the TARA declares for
`TS-ENG-001`.
