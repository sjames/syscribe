---
type: ADR
id: ADR-SIL-ARCH-001
name: "Diverse software development teams for Channel A and Channel B vital logic"
status: accepted
date: "2026-05-28"
tags:
  - software
  - diversity
  - common-cause-failure
  - EN50128
  - SIL4
---

## Context

SIL 4 requires the dangerous failure rate of vital software to be below 10⁻⁸ per hour. For software of the complexity of a full interlocking vital function (> 50,000 lines-of-equivalent logic, hundreds of states, thousands of route combinations), this target cannot be achieved by testing alone. The primary residual risk in a 2oo2D architecture (ADR-SIL-SYS-001) — where both channels execute software derived from the same specification — is a **systematic common-cause software failure**: a logic error present in both Channel A and Channel B that causes both channels to produce an identical but incorrect output. This failure mode evades the 2oo2D comparison mechanism entirely, because both channels agree on the (wrong) output.

EN 50128 §6.7.4 ("Diverse programming") provides a structured menu of techniques for reducing common-cause software failure probability. The highest-ranked techniques are:

- Different design teams (highest effectiveness rating)
- Different programming languages
- Different compilers
- Different operating system platforms

## Decision

Two entirely **independent development teams** shall implement the Channel A and Channel B vital logic from the same B-Method / Event-B formal specification (see ADR-SIL-SW-001):

| Attribute | Channel A | Channel B |
|---|---|---|
| Development team | Team Alpha (external supplier A) | Team Bravo (external supplier B) |
| Programming language | Ada 2012 / SPARK 2014 | C (MISRA-C:2012 subset) |
| Static analysis | GNAT Pro + GNATprove (SPARK proofs) | Frama-C value analysis (EVA plugin) |
| Compiler | GNAT Pro (AdaCore) | Green Hills MULTI (INTEGRITY) |
| RTOS platform | INTEGRITY RTOS (Green Hills) | LynxOS-SE (Lynx Software Technologies) |
| Target processor | Freescale MPC5674F (Channel A board) | NXP S32G274A (Channel B board) |

Both teams receive exactly the same version of the B-Method formal specification, frozen at each specification baseline release. Neither team has access to the other's implementation.

The formal specification serves as the **common oracle**: both implementations are compared against the specification, not against each other. This eliminates the risk that both teams share the same misunderstanding of the requirements.

## Rationale

- **EN 50128 highest-effectiveness defence**: §6.7.4 Table A.5 rates "different design teams" as the measure with the highest effectiveness rating against common-cause software failure. Combining it with language diversity, compiler diversity, and RTOS diversity provides defence-in-depth against all known systematic software failure modes.
- **Language diversity secondary benefit**: An Ada integer overflow is a bounded, defined behaviour (raising `Constraint_Error` and triggering the safe state via the Ada runtime). An identical logic error in C with MISRA-C constraints may cause undefined behaviour that wraps silently, producing a different output. This asymmetry means that a logic error that manifests as a specific numerical artefact in one language is unlikely to produce the same artefact in the other. The comparison mechanism will therefore reveal it.
- **SPARK proofs for Channel A**: SPARK 2014's proof mode provides formal verification that Channel A code is free of runtime errors (no buffer overflows, no integer overflow, no uninitialized reads, no null pointer dereferences) within the defined SPARK subset. This provides an additional layer of assurance beyond the B-Method specification proof.
- **Frama-C value analysis for Channel B**: The EVA (Evolved Value Analysis) plugin in Frama-C provides abstract interpretation over all possible execution paths of Channel B, guaranteeing absence of undefined behaviour in the MISRA-C subset. This is the C-language equivalent of SPARK proof.
- **Formal specification as oracle**: The B-Method specification (ADR-SIL-SW-001) is the definitive reference for both implementations. Any deviation from the specification is a defect in that implementation — it does not matter whether both implementations deviate in the same direction. The ISA reviews each implementation independently against the specification, eliminating the risk of mutual review between teams reinforcing shared errors.

## Consequences

- **Approximately 2× software development cost**: Two independent teams, two build chains, two RTOS platforms, two sets of integration testing. This is accepted as the necessary cost of demonstrating SIL 4 for software of this complexity. The alternative — a single-channel implementation with extensive testing — cannot demonstrate 10⁻⁸/h at the required confidence level.
- **Formal specification configuration management**: The B-Method specification is a shared artefact that drives both implementations. Its version control must be tightly governed: both teams must receive exactly the same specification revision, with no informal communications about interpretation. Specification change control is therefore a critical process: any clarification to the specification must be documented as a formal specification amendment and released to both teams simultaneously.
- **ISA dual-review burden**: The ISA must review both Channel A and Channel B implementations independently against the B-Method specification. This doubles the software review effort for the ISA. Assessor selection must confirm availability and competence for both Ada/SPARK and C/MISRA-C/Frama-C review.
- **Integration testing complexity**: The 2oo2D integration test must demonstrate that both channels produce identical outputs for the same inputs across the full interlocking combinatorial space. This requires a Hardware-in-the-Loop test bench capable of exercising all route combinations. Test infrastructure cost is significant.
- **Long-term maintenance**: Specification changes require re-implementation in both channels. Maintenance procedures must prohibit one-channel-only patching. An emergency patch process (with ISA oversight and expedited review) must be defined for in-service safety-critical defects.
