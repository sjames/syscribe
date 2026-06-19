---
type: Diagram
name: Sensor Reading Sequence
diagramKind: Sequence
pumlMode: companion
pumlFile: ./SensorReadingSeq.puml
subject: System::EngineECU
shapes:
  p-cps:  {ref: "System::Sensors::CrankshaftPositionSensor", kind: lifeline}
  p-tps:  {ref: "System::Sensors::ThrottlePositionSensor",   kind: lifeline}
  p-ecs:  {ref: "System::EngineControlSoftware",             kind: lifeline}
  p-sm:   {ref: "System::Software::SafetyMonitor",           kind: lifeline}
  p-tc:   {ref: "System::Software::ThrottleControl",         kind: lifeline}
  p-fc:   {ref: "System::Software::FuelControl",             kind: lifeline}
  p-ta:   {ref: "System::Actuators::ThrottleActuator",       kind: lifeline}
  p-fi:   {ref: "System::Actuators::FuelInjector",           kind: lifeline}
edges:
  e-m1:  {source: p-cps, target: p-ecs, kind: message, label: "crankPosition(angle, rpm)"}
  e-m2:  {source: p-tps, target: p-ecs, kind: message, label: "throttlePosition(pos1, pos2)"}
  e-m3:  {source: p-ecs, target: p-sm,  kind: message, label: "checkSafetyConstraints()"}
  e-m4:  {source: p-sm,  target: p-ecs, kind: return,  label: "safetyOK : bool"}
  e-m5:  {source: p-ecs, target: p-tc,  kind: message, label: "computeThrottleTarget(rpm, pos)"}
  e-m6:  {source: p-tc,  target: p-ta,  kind: message, label: "setPosition(target)"}
  e-m7:  {source: p-ecs, target: p-fc,  kind: message, label: "computeFuelInjection(rpm, lambda)"}
  e-m8:  {source: p-fc,  target: p-fi,  kind: message, label: "inject(pulseWidth)"}
---

![Sensor Reading Sequence](./SensorReadingSeq.svg)

10 ms control loop: crankshaft and throttle sensor readings arrive at the Engine
Control Software, the Safety Monitor validates constraints, and the Throttle and
Fuel Control sub-modules output commands to the actuators.
