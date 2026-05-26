---
type: ActionDef
name: LandingAction
supertype: Actions::Action
parameters:
  - name: descentRateMs
    typedBy: ScalarValues::Real
    direction: in
  - name: success
    typedBy: ScalarValues::Boolean
    direction: return
subActions:
  - name: commandDescent
    kind: SendAction
    payload: Items::ControlCommand
    via: controlOut
  - name: waitForGround
    kind: LoopAction
    loopKind: until
    condition: "self.altitudeM <= 0.1"
    body:
      - name: monitorDescent
        kind: AcceptAction
        payload: Items::GPSFix
        via: gpsIn
  - name: disarm
    kind: AssignmentAction
    target: self
    referent: armStatus
    value: "ArmStatus::disarmed"
    valueKind: bound
successionConnections:
  - after: commandDescent
    before: waitForGround
  - after: waitForGround
    before: disarm
---

Landing action commanding a controlled descent at a specified rate until touchdown, then disarming the motors.
