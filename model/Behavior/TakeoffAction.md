---
type: ActionDef
name: TakeoffAction
supertype: Actions::Action
parameters:
  - name: targetAltitudeM
    typedBy: ScalarValues::Real
    direction: in
  - name: success
    typedBy: ScalarValues::Boolean
    direction: return
subActions:
  - name: setThrottle
    kind: AssignmentAction
    target: self
    referent: throttlePercent
    value: "0.6"
    valueKind: initial
  - name: waitForAltitude
    kind: LoopAction
    loopKind: until
    condition: "self.altitudeM >= targetAltitudeM"
    body:
      - name: adjustThrottle
        kind: AssignmentAction
        target: self
        referent: throttlePercent
        value: "throttlePercent + 0.01"
        valueKind: bound
successionConnections:
  - after: setThrottle
    before: waitForAltitude
---

Takeoff action commanding the UAV to climb to a specified altitude. Incrementally adjusts throttle until the target altitude is reached.
