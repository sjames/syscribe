---
type: PartDef
name: Route Processor
domain: software
silLevel: 4
supertype: System::Software::VitalSoftwareBase
satisfies: []
features:
  - name: maxRoutes
    typedBy: ScalarValues::Integer
  - name: scanPeriodMs
    typedBy: ScalarValues::Integer
---

The Route Processor evaluates route setting and cancellation requests from the signaller workstation. A route request is accepted only after all interlocking conditions are satisfied simultaneously within a single scan cycle:

1. All track circuit sections on the route and overlap are clear (unoccupied).
2. No conflicting route is currently set or in the process of being set (as reported by ConflictChecker).
3. All points on the route are detected in the required position.
4. For routes that include a level crossing, the LX module reports barriers down.

When all conditions are met, the route is entered into the route table as a locked route. A locked route protects against:

- Points movement: any point locked by a set route cannot be moved until the route is released.
- Conflicting route setting: ConflictChecker references the route table to reject conflicting requests.
- Track circuit override: a track-circuit clear-override is only possible through a documented degraded-mode procedure, not through normal interlocking logic.

The route is automatically released when the train has been detected on the overlap and subsequently clears the last track section of the overlap (approach locking and overlap releasing). Manual release by the signaller is subject to time-lock procedures.

The RouteProcessor is implemented as a B-Method abstract machine (`RouteProcessor.mch`) with formal proof obligations covering all interlocking invariants. Proof obligations are discharged by Atelier-B. The generated C code is the only output used in production.
