---
type: TARASheet
id: TARA-SIL-001
name: "TARA — Railway CBI operator workstation and field bus interfaces"
status: approved
date: "2026-05-28"

damageTable:
  - id: DS-SIL-001
    name: "Unauthorised route set causes conflicting train movements — collision"
    damageSeverity: severe
    impactCategories: [safety, operational]
  - id: DS-SIL-002
    name: "Replay of field bus command moves points under a train — derailment"
    damageSeverity: severe
    impactCategories: [safety, operational]
  - id: DS-SIL-003
    name: "Denial of service on cross-comparison bus forces persistent safe state — service disruption"
    damageSeverity: major
    impactCategories: [operational, financial]

threatTable:
  - id: TS-SIL-001
    name: "Compromised maintainer workstation issues unauthorised route commands to vital processor via management LAN"
    attackFeasibility: medium
    attackVector: adjacent
    damageScenarios: [DS-SIL-001]
  - id: TS-SIL-002
    name: "Attacker with physical access to lineside cable replays captured valid field bus point-movement command"
    attackFeasibility: low
    attackVector: local
    damageScenarios: [DS-SIL-002]
  - id: TS-SIL-003
    name: "Flood attack on cross-comparison Ethernet bus saturates link — comparison messages lost"
    attackFeasibility: medium
    attackVector: adjacent
    damageScenarios: [DS-SIL-003]

goalTable:
  - id: CSG-SIL-001
    name: "Ensure authenticity and authorisation of all operator commands reaching the vital processor"
    calLevel: CAL3
    securityProperty: authenticity
    threatScenarios: [TS-SIL-001]
  - id: CSG-SIL-002
    name: "Ensure integrity and freshness of all field bus commands to object controllers"
    calLevel: CAL3
    securityProperty: integrity
    threatScenarios: [TS-SIL-002, TS-SIL-003]

controlTable:
  - id: SC-SIL-001
    name: "Mutual TLS 1.3 authentication between operator workstation and vital processor management interface, with hardware-bound client certificates"
    controlType: prevention
    implementsGoals: [CSG-SIL-001]
  - id: SC-SIL-002
    name: "EN 50159 Category 2 safety codes on all field bus messages prevent replay and insertion attacks"
    controlType: prevention
    implementsGoals: [CSG-SIL-002]
  - id: SC-SIL-003
    name: "Dedicated VLAN with rate limiting on cross-comparison bus; comparison bus loss triggers immediate safe state (availability vs. safety trade-off)"
    controlType: detection
    implementsGoals: [CSG-SIL-002]
---

## Scope

This TARA covers the external attack surfaces of the Computer-Based Interlocking (CBI):

1. **Operator workstation LAN** — the management network connecting signaller workstations (in the signal box) to the vital processor management interface. This network also carries maintainer laptops, condition monitoring systems, and the event logging server.
2. **Field bus to object controllers** — the industrial Ethernet field bus connecting the vital processor output stage to object controllers (point machines, signal heads, track circuit detector interfaces) in lineside equipment rooms and cabinets. Cable lengths of 50–500 m; physical access cannot be fully controlled.
3. **Cross-comparison Ethernet bus** — the dedicated network segment carrying 2oo2D comparison frames between Channel A and Channel B vital processors. This segment is physically within the interlocking equipment room (IER) and is protected by physical access control to the IER.

## Out of Scope

The vital logic itself (the B-Method specification and its implementation in both channels) is not network-accessible. The vital processor has no IP stack on its safety execution path. Its management interface is connected to the management LAN via a one-way data diode for outgoing monitoring traffic only. Inbound configuration updates require physical access to the interlocking room and a keyed maintenance mode switch; these are covered by physical security procedures, not network security controls.

## Regulatory Context

- **NIS2 Directive (EU Directive 2022/2555)**: Railway operators and infrastructure managers are identified as critical infrastructure entities. They must implement cybersecurity risk management measures proportionate to the risk. This TARA is the primary risk management artefact for the CBI.
- **ERA ERTMS Cybersecurity Regulation (EU 2016/919)**: Requires that ERTMS trackside and on-board equipment implement cybersecurity measures. The CBI management interface interacts with ERTMS via the Radio Block Centre (RBC); the security controls defined here are consistent with ERA cybersecurity requirements.
- **EN 50159:2010**: The communication safety standard. EN 50159 Category 2 (threat scenario: open transmission system, intentional attack) applies to both the field bus and the cross-comparison bus. See ADR-SIL-COMM-001 for the architectural decision.

## Attack Feasibility Ratings

Attack feasibility is rated on the ISO/SAE 21434 scale:

| Rating | Elapsed time | Specialist knowledge | Resources | Description |
|---|---|---|---|---|
| `low` | > 6 months | Domain expert + special tools | State actor | Requires nation-state capability |
| `medium` | 1–6 months | Domain expert | Organised group | Skilled attacker with railway domain knowledge |
| `high` | < 1 month | Practitioner | Individual | Commercially available tools |
| `veryHigh` | < 1 week | Layperson | — | Script-kiddie attack |

TS-SIL-001 is rated `medium` because compromising a maintainer workstation requires obtaining credentials or exploiting a vulnerability in the workstation software — feasible by an organised group with targeted phishing and knowledge of the railway management network.

TS-SIL-002 is rated `low` because physical access to a lineside cable, plus knowledge of the EN 50159 frame format sufficient to capture and replay a valid command, requires both domain expertise and physical proximity. The category 2 sequence counter further reduces the replay window to a single 20 ms cycle, making successful replay of a captured message extremely difficult.

TS-SIL-003 is rated `medium` because flooding an Ethernet segment requires only network access — achievable via a compromised device on the management VLAN if VLAN isolation fails, or via a compromised object controller that has access to the field bus.
