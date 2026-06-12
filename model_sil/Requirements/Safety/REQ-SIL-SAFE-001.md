---
type: Requirement
id: REQ-SIL-SAFE-001
name: Conflict checker shall prevent any two conflicting routes from being simultaneously set
status: approved
reqDomain: software
silLevel: 4
verificationMethod: analysis
derivedFrom:
  - REQ-SIL-SAFE-000
breakdownAdr: ADR-SIL-SYS-001
derivedFromSafetyGoal: SG-SIL-001
---

The ConflictChecker SWC **shall** maintain a formally-verified conflict matrix covering all route pairs in the station layout. For every route request, the ConflictChecker **shall** evaluate all conflict conditions before returning a "route clear to set" response to the RouteProcessor. A conflict condition exists if the requested route shares any of: track section, points machine, flank protection zone, or overlap region with any currently-set or partially-set route. The ConflictChecker **shall** use a fail-safe default: any ambiguity or evaluation error shall return "route blocked". The conflict matrix **shall** be specified as a B-Method abstract machine with invariant proofs, and verified by model checking against the track topology (REQ-SIL-SW-003). The implementation shall be produced by two diverse teams and cross-validated (REQ-SIL-SW-002).
