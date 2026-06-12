---
type: ADR
id: ADR-SIL-SW-001
name: "Use B-Method (Event-B) for formal specification of all vital interlocking logic"
status: accepted
date: "2026-05-28"
tags:
  - software
  - formal-methods
  - EN50128
  - SIL4
---

## Context

EN 50128 Table A.9 rates formal methods as "highly recommended" (HR) at SIL 3 and SIL 4. For the vital interlocking logic — a finite state machine of approximately 50,000 lines-of-equivalent-logic — informal or semi-formal approaches cannot demonstrate the absence of specification errors with sufficient confidence at SIL 4. The following candidate approaches were evaluated:

**Option A — Informal natural-language specification**: Lowest entry cost; highest residual ambiguity. Widely used at SIL 1/2 but provides no mechanically-checkable proof that the specification is internally consistent or free from conflicting route conditions. Not adequate for SIL 4 vital logic.

**Option B — Semi-formal structured text / tabular notation (SFN tables)**: Reduces ambiguity through structure; supports automated consistency checking of individual tables. However, the global invariant (no two conflicting routes simultaneously set) cannot be proved at the specification level — it must be checked by exhaustive testing over all route combinations, which is infeasible for a large interlocking.

**Option C — Formal specification using Z notation**: Expressive, mathematically rigorous, and well-understood. However, Z lacks a built-in refinement calculus: the translation from Z specification to implementation code is manually verified, reintroducing the risk of specification-to-code errors at the most safety-critical step. The Z community in the railway sector is small, limiting assessor availability.

**Option D — Formal specification using B-Method / Event-B with Atelier-B and ProB**: Provides a mechanically-checked refinement chain from the abstract machine (safety invariants) through multiple refinement levels to the implementation. Used for the Paris Météor (Line 14) driverless metro interlocking — the first industrial application of formal methods to SIL 4 railway signalling.

## Decision

Adopt **B-Method / Event-B**, with proofs discharged using **Atelier-B** and model-checking counterexample generation using **ProB**. The vital interlocking logic shall be specified as a set of abstract machines (one per interlocking function: route setting, route locking, point control, signal control, track circuit occupation). Each machine carries invariants that are proved once at the abstract level and inherited by all refinements.

## Rationale

- **Mechanically-checked refinement**: B-Method's refinement calculus gives a fully machine-checked path from the abstract safety invariants (e.g., "no two conflicting routes shall be simultaneously set") to the concrete implementation. Changes to the implementation trigger re-proof only of the affected refinement steps, not of the top-level invariants.
- **Railway precedent and assessor acceptance**: The Météor Line 14 interlocking established B-Method as an accepted approach for SIL 4 railway signalling in Europe. Atelier-B has an established qualification record with railway certification bodies (CERTIFER, TÜV, Bureau Veritas). This is a significant benefit over alternatives with less evidence of assessor acceptance.
- **Event-B refinement for incremental development**: Event-B's super-machine / sub-machine structure allows the interlocking invariants to be proved once for the most abstract model; all subsequent refinements inherit the proof. Adding new routes or modifying guard conditions requires re-verifying only the affected events, not the full invariant corpus.
- **ProB model checking**: ProB provides symbolic model-checking over the B-Method abstract machines, generating counterexamples for erroneous guard conditions. This catches errors that pass proof obligation checking but are operationally wrong — a complementary technique that reduces proof burden.
- **Integration with existing EN 50128 V-model**: The B-Method specification serves as the Software Requirements Specification (SRS) artefact in the EN 50128 development lifecycle. Code generated from refined machines (or manually implemented against the specification) can be verified by back-translation to the B machines.

## Consequences

- **Specialist skills required**: B-Method is not a standard software engineering skill. Engineers require dedicated training (typically 2–4 weeks for an experienced software engineer; 6–8 weeks to reach assessable competency). Onboarding time for new engineers is longer than for conventional approaches.
- **Proof obligation corpus maintenance**: Every change to the specification regenerates proof obligations. The project must maintain a proof-obligation management process: all obligations must be discharged before any specification revision is released to the development teams. This is a continuous engineering activity throughout the project.
- **Complex timing properties not provable in B-Method alone**: B-Method is not a real-time formalism. Timing requirements (e.g., signal aspect changes within N cycles of route release) require separate WCET analysis (see Requirements/Software for timing requirements). The formal specification covers logical correctness; timing is covered by the hardware platform analysis.
- **Limited assessor pool**: The ISA (Independent Safety Assessor) must be competent to review B-Method proofs. This limits the pool of acceptable assessors. Assessor selection must account for this constraint at project initiation.
- **Tool qualification**: Atelier-B must be assessed under EN 50128 §6.7.4 as a tool used in the development of SIL 4 software. A T3 tool qualification argument is required (or the tool must be operated in a mode where its outputs are independently verified).
