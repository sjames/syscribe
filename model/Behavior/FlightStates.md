---
type: StateDef
name: FlightStates
supertype: States::StateAction
isParallel: false
subStates:
  - name: disarmed
    isInitial: true
    transitions:
      - target: armed
        accept:
          payload: Items::ControlCommand
        guard: "armStatus == ArmStatus::disarmed and gpsReceiver.fixQuality >= 1"
  - name: armed
    transitions:
      - target: takingOff
        accept: Items::ControlCommand
        guard: "armStatus == ArmStatus::armed"
        effect:
          name: startTakeoff
          typedBy: Behavior::TakeoffAction
      - target: fault
        accept:
          payload: Items::ControlCommand
        guard: "imu.accelXMs2 > 30.0"
  - name: takingOff
    entryAction: Behavior::TakeoffAction
    transitions:
      - target: flying
        guard: "altitudeM >= targetAltitudeM"
      - target: fault
        guard: "altitudeM < 0.1 and elapsedS > 10.0"
  - name: flying
    doAction: Behavior::MissionExecution
    transitions:
      - target: landing
        accept: Items::ControlCommand
        guard: "command.commandType == FlightMode::returnToHome"
        effect: Behavior::LandingAction
      - target: fault
        guard: "batteryPct < 10.0"
  - name: landing
    entryAction: Behavior::LandingAction
    transitions:
      - target: disarmed
        guard: "altitudeM <= 0.1"
      - target: fault
        guard: "descentRateMs > 3.0"
  - name: fault
    isFinal: false
    entryAction: Behavior::LandingAction
    transitions:
      - target: disarmed
        accept: Items::ControlCommand
        guard: "groundOperatorCleared"
---

Flight state machine governing UAV operational states from disarmed through armed, takeoff, flying, and landing, with fault handling at each stage.
