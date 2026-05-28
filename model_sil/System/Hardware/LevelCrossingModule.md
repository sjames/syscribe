---
type: PartDef
name: Level Crossing Module
domain: hardware
features:
  - name: barrierCount
    typedBy: ScalarValues::Integer
  - name: warningTimeS
    typedBy: ScalarValues::Integer
---

The Level Crossing Module controls road-side barriers and warning lights for a level crossing (LX). It operates under the authority of the vital processor: a barrier lowering command is issued before any conflicting train movement signal is cleared.

The module sequences the warning phase (flashing lights and audible alarm, duration at least warningTimeS seconds) before initiating barrier descent. Barriers are confirmed down by limit switches independently monitored by the module. The vital processor receives a barrier-down confirmation signal before it may authorise signal clearance for any route that conflicts with the crossing.

If a barrier fails to reach the down position within the configured timeout, or if a barrier-up command arrives without a corresponding down-confirmed state, the module reports a fault to the vital processor. The vital processor treats this as a pre-condition failure for signal clearance; the affected signal remains at the most-restrictive aspect.

The module also monitors the road occupancy detector (where fitted) for pedestrians or stalled vehicles on the crossing. Detection of road occupancy while a conflicting train route is set causes an emergency cancellation request to the vital processor.

The LX module is designed to EN 50159 Category 2 for communication with the vital processor and meets SIL 3 integrity requirements for the level crossing protection function.
