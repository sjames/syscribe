---
type: PartDef
name: Points Controller
domain: software
silLevel: 4
supertype: System::Software::VitalSoftwareBase
satisfies:
  - REQ-SIL-SAFE-003
features:
  - name: pointsCount
    typedBy: ScalarValues::Integer
  - name: moveTimeoutMs
    typedBy: ScalarValues::Integer
---

The Points Controller issues movement commands to Points Drive Modules and supervises the resulting detection confirmations. The controller enforces the following vital rules:

**Command authority.** A points movement command is only issued if the RouteProcessor has locked the points for the requested move via the route table. Any attempt to move points that are currently locked by a set route is rejected and logged.

**Detection supervision.** After commanding a move, the controller monitors the detection feedback from the PointsDriveModule. The detection must be received and held continuously for the debounce period within the moveTimeoutMs window. If detection is not confirmed within the timeout, the points are placed in a fail-safe locked state: they are reported to the RouteProcessor as being in an unknown position, and no signal authorised over those points will be cleared until the locked state is resolved by a maintenance operator.

**Simultaneous detection fault.** If the PointsDriveModule reports both normal and reverse detection contacts simultaneously (physically impossible in a correctly functioning machine), the PointsController enters a diagnostic fault state for those points and reports to the vital processor's safe state monitor.

**Atomic read.** Point detection states are read atomically at the start of each scan cycle and held constant throughout the cycle to avoid races between point movement and route condition evaluation.
