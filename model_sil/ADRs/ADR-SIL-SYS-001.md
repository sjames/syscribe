---
type: ADR
id: ADR-SIL-SYS-001
title: "Use 2oo2D voting architecture rather than 2oo3 for vital processing"
status: accepted
date: "2026-05-28"
tags:
  - architecture
  - voting
  - SIL4
  - EN50129
---

## Context

EN 50129 requires SIL 4 integrity for the vital processing of a Computer-Based Interlocking. The following candidate architectures were evaluated:

- **1oo1** — Single channel, lowest hardware cost, but cannot achieve SIL 4 hardware failure rate (PFH < 10⁻⁸/h) without an unrealistically low component failure rate.
- **1oo2D** — One active channel plus one independent diagnostic channel. The diagnostic channel can force the safe state but does not participate in normal output generation. Achieves SIL 3 without additional arguments; SIL 4 requires additional proof obligations.
- **2oo2** — Two active channels; both must agree to generate an output. A single channel failure forces the safe state. High diagnostic coverage possible but no independent comparison mechanism between channels.
- **2oo2D** — Two independent vital channels with a cross-comparison bus. Each channel independently drives a fail-safe output relay. Any disagreement detected on the comparison bus immediately forces the safe state via de-energised relay output. The diagnostic subsystem monitors both channels continuously.
- **2oo3** — Three active channels; majority vote drives the output. One channel can fail without loss of service (high availability). The highest hardware cost and the most complex common-cause failure argument of all candidates.

Industry precedent: Siemens Sinet, Bombardier Interflo 200, and Thales PIPC all use 2oo2D or a minor variant of it for EN 50129 SIL 4 interlocking vital processing.

## Decision

Adopt **2oo2D** — two independent vital channels, each capable of independently forcing the safe state, with fail-safe relay output architecture and a dedicated cross-comparison bus between the channels.

The two channels (Channel A and Channel B) are physically and electrically independent. They receive the same inputs from field elements, execute the same vital interlocking logic in parallel, and compare their outputs on the cross-comparison bus each execution cycle (20 ms). Any comparison disagreement triggers both channels to independently de-energise their output relays, placing all signals at danger and all point machines in the last-known safe position.

## Rationale over 2oo3

- **Safety over availability**: 2oo3 gives higher availability — one channel can fail without loss of service — but requires a more complex common-cause failure argument. For a railway interlocking, a safe-side failure (all signals to red) is the expected response to any doubt; an unsafe failure is catastrophic and unacceptable. The safety case benefits from the simpler 2oo2D argument.
- **Highest diagnostic coverage**: 2oo2D achieves the highest diagnostic coverage for dangerous failures of any two-channel architecture. Any undetected single-channel error — whether hardware or software — produces a disagreement on the cross-comparison bus and is immediately revealed. The comparison bus itself is the diagnostic mechanism.
- **Reduced common-cause failure exposure**: Maintaining true independence (hardware, power supply, software implementation, development team) between two channels is simpler than maintaining independence between three. Each additional channel introduces additional interfaces and coupling opportunities.
- **Precedent and tooling**: 2oo2D has an established track record at SIL 4 in the railway domain, with accepted assessor methodologies and IEC 62280-compliant tooling for PFH calculation.

## Consequences

- **Reduced availability vs 2oo3**: Loss of one channel forces the safe state — the interlocking goes to red and remains there until the faulty channel is restored. For a major junction this represents a service disruption. This trade-off is accepted: railway operational procedures include degraded-mode working and manual operation.
- **High DC requirement**: The 2oo2D architecture only achieves SIL 4 hardware failure rate if diagnostic coverage (DC) of the per-channel hardware is ≥ 99%. The detailed hardware design must demonstrate this via FMEA.
- **Cross-comparison bus integrity**: The comparison bus must itself achieve SIL 4 integrity; otherwise it is the weakest link. EN 50159 Category 2 safety codes are applied to all comparison bus messages (see ADR-SIL-COMM-001).
- **Software diversity**: Because both channels execute the same logic, a common-cause software fault present in both would not be revealed by the comparison. This risk is mitigated by the decision documented in ADR-SIL-ARCH-001 (diverse development teams and implementation languages).
