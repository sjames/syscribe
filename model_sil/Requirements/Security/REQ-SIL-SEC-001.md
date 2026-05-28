---
type: Requirement
id: REQ-SIL-SEC-001
title: "Operator commands shall be authenticated before reaching the vital processor"
status: approved
reqDomain: system
verificationMethod: inspection
derivedFrom:
  - REQ-SIL-SYS-000
breakdownAdr: ADR-SIL-COMM-001
derivedFromSecurityGoal: CSG-SIL-001
silLevel: 4
tags:
  - security
  - authentication
  - operator-interface
---

All route request and cancellation commands originating from the signaller workstation **shall** be authenticated by the vital processor management interface using mutual TLS 1.3 with hardware-bound client certificates stored in the workstation's TPM 2.0 module before any command processing occurs. Any command arriving without a valid, current TLS session **shall** be discarded without processing, and a security alarm **shall** be raised to the maintainer workstation log.

The vital processor **shall** not expose any unauthenticated management interface, either on the operator LAN or any other network segment. The management interface **shall** operate exclusively over the authenticated TLS channel.

Command authorisation — determining which routes a given authenticated operator is permitted to set or cancel — **shall** be enforced by an Access Control List (ACL) maintained within the vital processor itself. Authorisation decisions **shall** not be delegated to the operator workstation or any other external component. The ACL **shall** be configurable only by an authenticated system administrator via the same TLS channel.

The implementation shall satisfy security control SC-SIL-001.

## Rationale

The threat scenario TS-SIL-001 (TARA-SIL-001) identifies the compromised maintainer workstation as the primary network-based attack vector for unauthorised route setting. Without authentication, any device on the management LAN could issue route commands. Mutual TLS with TPM-bound certificates prevents a compromised workstation from issuing commands using stolen credentials stored in software — the private key never leaves the TPM.

Separating authentication (TLS) from authorisation (vital processor ACL) ensures that even a compromised workstation presenting valid credentials cannot exceed its configured route permissions. The vital processor is the authoritative policy enforcement point because it is the safety-critical element that acts on the command.

## Notes

- Verification method `inspection` covers code review of the TLS handshake implementation and ACL enforcement logic, plus review of the certificate management process documentation.
- The TLS session **shall** use TLS 1.3 exclusively. TLS 1.2 and earlier **shall** be disabled. Only cipher suites with PFS (ECDHE key exchange) **shall** be enabled.
- Certificate lifecycle (issuance, renewal, revocation) is covered by the PKI management plan, which is a separate document referenced from the Security Case.
- Reference standard: ETSI EN 319 401 (Electronic Signatures and Infrastructures — General Policy Requirements for Trust Service Providers).
