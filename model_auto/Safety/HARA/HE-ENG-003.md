---
type: HazardousEvent
id: HE-ENG-003
title: Engine over-speed due to failed closed throttle during downshift
status: approved
severity: S2
exposure: E2
controllability: C3
operationalSituation: Driver decelerating from highway speed with engine braking
---

During a manual or automated downshift, the ECU issues a deceleration fuel-cut
command and expects the throttle plate to return to the idle stop. If the throttle
plate fails to close — due to a mechanical jam or a stuck-open actuator fault —
excess air continues to enter the cylinders. When the fuel-cut lift-off condition
is released (e.g. re-engaging drive), the combination of high air mass and
ignition timing advance can drive engine speed above the rev limiter threshold,
potentially causing mechanical over-speed damage and loss of engine torque control.

## Hazard description

Contributing factors:

- Throttle plate fails to close on deceleration fuel-cut command (actuator fault
  or return spring weakened)
- Residual or re-enabled fuel delivery with high manifold air mass
- Ignition timing not retarded independently of throttle position feedback
- Engine speed climbs above 6500 rpm — the calibrated rev limiter threshold

The primary hazard is mechanical engine damage (over-speed beyond the design
limit of the crankshaft and connecting rods) and secondary loss of controlled
torque delivery during a transient manoeuvre. The driver can mitigate by
depressing the clutch or selecting neutral to break the driveline torque path.

## Risk parameters

ASIL = S2 × E2 × C3 = **ASIL A**

- **S2** — injury possible (engine over-speed can cause sudden loss of torque or
  component ejection in extreme cases), but not life-threatening in most scenarios
- **E2** — moderate exposure: occurs during gear changes, which happen regularly
  during highway driving but not continuously
- **C3** — controllable: driver can depress the clutch or select neutral within
  the 200 ms FTTI window; automated transmissions have additional override paths
