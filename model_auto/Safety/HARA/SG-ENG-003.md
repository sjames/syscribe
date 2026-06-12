---
type: SafetyGoal
id: SG-ENG-003
name: Prevent engine over-speed above rev limiter threshold
status: approved
asilLevel: A
safeState: Fuel cut and ignition retard enforced above 6500 rpm
ftti: 200ms
hazardousEvents:
  - HE-ENG-003
---

The Engine ECU **shall** prevent engine over-speed above the calibrated rev
limiter threshold (6500 rpm) regardless of throttle position sensor feedback, by
enforcing an independent fuel-cut and ignition-retard mechanism whenever the
crankshaft speed exceeds this threshold.

## Rev limiter safety function

The rev limiter is implemented as a two-stage mechanism:

1. **Soft limiter** — ignition retard applied above 6200 rpm, reducing torque
   output and providing early warning of approaching the hard limit. This stage
   uses ignition angle retard (up to −30° BTDC) independently of the throttle
   control loop.
2. **Hard limiter** — fuel cut enforced above 6500 rpm. Each cylinder's injection
   pulse is suppressed on a sequential-cylinder basis to avoid abrupt torque steps.
   The fuel cut is triggered directly from the crank position event counter and
   does not depend on throttle position feedback.

Both stages operate in the safety monitor execution context and are independent
of the throttle control PID loop, ensuring that a throttle position sensor fault
or a stuck-open throttle plate cannot defeat the over-speed protection.

## Safe state definition

The safe state is fuel cut combined with ignition retard above 6500 rpm.
This brings engine speed below the threshold within the 200 ms FTTI. No limp-home
throttle position is required — the rev limiter operates purely on injection and
ignition scheduling.

## Independence from throttle position

The critical independence requirement is that the rev limiter reads engine speed
directly from the crankshaft position sensor event counter, not from any throttle
or pedal demand signal. A stuck-open throttle plate delivering excess air is
therefore not able to prevent the fuel-cut from executing.

## ASIL target

ASIL A — derived from HE-ENG-003 (S2 × E2 × C3). The safety mechanism is
allocated to the safety monitor software (REQ-ENG-SAFE-004) and supplemented by
throttle-close verification (REQ-ENG-SAFE-005).
