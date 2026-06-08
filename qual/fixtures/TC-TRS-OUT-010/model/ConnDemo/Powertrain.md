---
type: PartDef
name: Powertrain
features:
  - name: battery
    typedBy: ConnDemo::Battery
    multiplicity: "1"
  - name: motor
    typedBy: ConnDemo::Motor
    multiplicity: "1"
connections:
  - typedBy: ConnDemo::PowerConnectionDef
    ends:
      - end: source
        binds: battery.powerOut
      - end: receiver
        binds: motor.powerIn
---

Parent assembly wiring the battery's power output to the motor's power input.
