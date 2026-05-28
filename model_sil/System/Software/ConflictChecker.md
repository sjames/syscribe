---
type: PartDef
name: Conflict Checker
domain: software
silLevel: 4
supertype: System::Software::VitalSoftwareBase
satisfies:
  - REQ-SIL-SAFE-001
features:
  - name: conflictMatrixSize
    typedBy: ScalarValues::Integer
---

The Conflict Checker maintains a static conflict matrix derived from the track layout data loaded at system initialisation. For every ordered pair of routes (A, B), the matrix encodes one of three states:

- **Conflicting** — routes share track sections, share points, or cross paths. Both routes cannot be set simultaneously.
- **Compatible** — routes do not conflict; both may be set concurrently.
- **Sequential** — routes share an overlap; they may be set sequentially under approach locking rules.

The conflict matrix is consulted on every route setting request from the RouteProcessor and on every scan cycle for all currently-set routes. If a conflict is detected between a requested route and any currently-set route, the request is rejected and a diagnostic event is logged.

The conflict matrix is a formally-proven invariant of the interlocking: it is derived from the geographic layout at commissioning time and cannot be overridden by any operator action during normal operation. Changes to the conflict matrix require a full re-commissioning and re-acceptance test cycle.

The matrix is stored in non-volatile memory and is verified by a CRC check at system startup. Any CRC failure prevents the system from entering the operational state.

The ConflictChecker is implemented as a B-Method abstract machine with formal proof obligations verifying that the matrix is symmetric (A conflicts with B if and only if B conflicts with A) and complete (every route pair is classified).
