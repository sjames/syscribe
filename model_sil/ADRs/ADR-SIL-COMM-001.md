---
type: ADR
id: ADR-SIL-COMM-001
title: "Implement EN 50159 Category 2 safety codes on all vital communication paths"
status: accepted
date: "2026-05-28"
tags:
  - communication
  - EN50159
  - cybersecurity
  - SIL4
---

## Context

The CBI must communicate on two categories of vital path:

1. **Cross-comparison bus** — carries comparison frames between Channel A and Channel B vital processors every 20 ms execution cycle. These frames carry the output state computed by each channel; disagreement triggers the safe state.
2. **Field bus** — carries control commands from the vital processors to object controllers (point machines, signal heads, track circuit interfaces) over industrial Ethernet. Cable runs of hundreds of metres through trackside environments with significant EMC exposure and, critically, potential for physical access by both authorised and unauthorised personnel.

Both paths share a common 1 Mbit/s industrial Ethernet infrastructure with non-vital diagnostic traffic (maintainer workstation, event logging, condition monitoring). The network is therefore an **open transmission system** by the definition in EN 50159:2010 §3.1.2 — it cannot be guaranteed that only intended participants are connected.

EN 50159 defines two categories of safety code:

- **Category 1** — defences against unintentional errors arising from the transmission system itself (electromagnetic interference, hardware faults, signal degradation). Includes CRC and sequence number for corruption detection and message order verification.
- **Category 2** — additionally defends against intentional adversarial intervention: message insertion by an attacker with access to the network, replay of captured valid messages, and masquerading (a non-vital device presenting itself as a vital one using a wrong source address). Category 2 adds source/destination address checking and a timestamp to the Category 1 codes.

## Decision

Apply **EN 50159 Category 2** safety codes to all vital message paths: the cross-comparison bus and the field bus to object controllers.

The safety code frame for each vital message shall include:
- 32-bit CRC (polynomial TBD — minimum CRC-32/AUTOSAR for residual error probability ≤ 10⁻⁹ per message at maximum message length)
- 32-bit monotonically-increasing sequence counter (per source–destination pair, wrapping only after 2³² messages — approximately 95 days at 20 ms cycle time)
- 32-bit timestamp (milliseconds since last synchronisation epoch, derived from IEEE 1588 PTP)
- 16-bit source address (vital channel identifier: A or B, plus object controller index)
- 16-bit destination address

Total overhead: 14 bytes per message. At 20 ms cycle time and typical message body of 64–256 bytes, this represents less than 10% overhead on the 1 Mbit/s field bus.

## Rationale over Category 1

- **Open network threat model**: The field bus traverses the railway trackside environment. Physical access by an attacker who has compromised a lineside cabinet is a credible threat scenario for a safety-critical railway system (see TARA-SIL-001). Category 1 defends only against unintentional corruption from the transmission medium; it provides no defence against a replay attack by an attacker who has captured a valid point-movement command.
- **NIS2 and ERA cybersecurity obligations**: The CBI is a critical infrastructure component under the EU NIS2 Directive (Directive 2022/2555), and ERTMS-based signalling is subject to EU Regulation 2016/919 cybersecurity requirements. The threat of intentional attack is therefore a regulatory obligation, not merely a design consideration. Category 2 codes are the minimum necessary to address this obligation at the safety-protocol level.
- **Marginal cost**: Category 2 adds 14 bytes overhead to a message that already carries at minimum a 4-byte CRC under Category 1. This overhead is negligible on the 1 Mbit/s field bus and has no impact on cycle-time compliance.
- **Dual safety/security benefit**: The sequence number and timestamp fields that defend against replay attacks are also directly useful for the safety argument: the sequence number detects lost or out-of-order messages; the timestamp detects messages from a node whose clock has drifted (an early indicator of hardware failure). Category 2 therefore strengthens both the safety argument and the cybersecurity argument simultaneously.
- **Future-proofing for Category 3**: Category 3 adds a MAC (Message Authentication Code) requiring keyed cryptography. By including 16 bytes of reserved space in the protocol header (leaving room for a MAC field without changing the frame format), this decision preserves the option of upgrading to Category 3 if the threat model demands it in future. Key management infrastructure (HSM in each vital module, key distribution protocol) is noted as a future work item.

## Consequences

- **Clock synchronisation requirement**: The 32-bit timestamp requires all nodes on the vital network to maintain synchronisation to better than ±2 ms. This shall be achieved using IEEE 1588 Precision Time Protocol (PTP) over the same Ethernet infrastructure. The vital processors shall act as Boundary Clocks; a Grandmaster Clock with GPS input provides the primary time reference.
- **Cross-comparison bus protection**: Applying Category 2 to the cross-comparison bus (as well as the field bus) provides an additional layer of protection against a sophisticated attack that targets the comparison mechanism directly — forcing both channels to see identical (incorrect) comparison results. The sequence counter and timestamp make this attack infeasible without real-time access to the comparison bus.
- **Protocol header version field**: A 4-bit protocol version field is included in the header to allow in-field evolution of the safety code scheme (e.g., adding a MAC without re-flashing both channels simultaneously).
- **Object controller authentication**: Category 2 does not provide mutual authentication — an object controller cannot verify that the command it received was originated by the genuine vital processor. For the current threat model, the source address and physical security of the field bus cable are considered adequate. If the threat model is reassessed to include a compromised object controller generating false ACKs, Category 3 with per-device keys would be required.
