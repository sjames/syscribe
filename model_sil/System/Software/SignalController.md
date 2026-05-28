---
type: PartDef
name: Signal Controller
domain: software
silLevel: 4
supertype: System::Software::VitalSoftwareBase
satisfies:
  - REQ-SIL-SAFE-002
  - REQ-SIL-SAFE-004
features:
  - name: signalCount
    typedBy: ScalarValues::Integer
---

The Signal Controller manages signal aspect outputs via the Signal Output Modules. A signal may only be cleared (displayed at a non-red aspect) when all of the following conditions are simultaneously true within the same scan cycle:

1. **Route set and locked.** The RouteProcessor confirms that a route ending at this signal is in the locked state in the route table.
2. **Route and overlap clear.** All track circuit sections on the route and its overlap are reported unoccupied by the TrackCircuitInterface modules.
3. **Points in correct position.** All points on the route and overlap are reported by the PointsController as detected in the required position.
4. **Level crossing barriers down.** For any level crossing located on the route, the LevelCrossingModule reports barriers confirmed down.

Any condition becoming false while a signal is in a cleared state causes an immediate return to the most-restrictive aspect (red) within one scan cycle. The signal cannot be re-cleared until all conditions are re-established from scratch.

The SignalController does not store state about a signal's "clearing history" — each scan cycle is a complete re-evaluation of all clearing conditions from the current input vector. This ensures that no accumulated state error can cause an unsafe clearance.

Aspect selection is translated into relay commands sent to the SignalOutputModule. The controller reads back the relay feedback to confirm that the commanded aspect is displayed. A discrepancy between commanded aspect and confirmed feedback triggers a diagnostic failure report and causes the signal to be commanded to red on the next scan cycle.
