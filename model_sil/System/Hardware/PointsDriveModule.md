---
type: PartDef
name: Points Drive Module
domain: hardware
features:
  - name: strokeMs
    typedBy: ScalarValues::Integer
  - name: detectVoltage
    typedBy: ScalarValues::Real
    unit: V
---

The Points Drive Module drives a points machine to a commanded position (normal or reverse) and reads the resulting detection contacts independently from the drive circuit. Physical separation of the drive and detection paths is a fundamental safety requirement: the vital processor cannot confirm a points position from the drive current alone.

Position is confirmed only when the appropriate detection contacts are made and held for the required debounce period. The vital processor will not clear any signal whose route requires a specific points position unless confirmed detection has been received and acknowledged within the current scan cycle.

After commanding a move, the drive module monitors the elapsed time against the configured stroke timeout (strokeMs). A timeout without detection confirmation is treated as a points failure: the module reports the points as being in an unknown position, and the vital processor places all routes through those points in a locked-out state until maintenance clearance is received.

The detection circuit operates at a low DC voltage (detectVoltage) that is insufficient to drive the points motor, providing electrical isolation between detection and drive. Both the normal and reverse detection circuits are monitored simultaneously; simultaneous assertion of both contacts is a diagnostic failure condition.
