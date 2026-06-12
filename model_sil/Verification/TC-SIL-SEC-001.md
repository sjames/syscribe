---
type: TestCase
id: TC-SIL-SEC-001
name: Integration — Operator commands rejected without valid mTLS session
status: active
testLevel: L4
verifies:
  - REQ-SIL-SEC-001
---

```gherkin
Feature: Operator route commands require authenticated TLS 1.3 session

  Scenario: Unauthenticated route command is discarded without processing
    Given the vital processor management interface is listening on the operator LAN
    And no active TLS 1.3 session exists from the test client to the vital processor
    When the test client sends a raw TCP route-set command for route R1
    Then the vital processor shall reject the connection at the TLS handshake layer
    And no route processing shall occur
    And a security alarm event shall be written to the maintainer workstation log within 500 ms

  Scenario: Route command with revoked certificate is rejected
    Given a TLS client certificate that has been added to the certificate revocation list (CRL)
    When the operator workstation attempts to establish a TLS 1.3 session using the revoked certificate
    Then the vital processor shall reject the TLS handshake with alert certificate_revoked
    And no management session shall be established
    And the rejection shall be logged with the certificate serial number

  Scenario: Route command on authenticated TLS 1.3 session is processed
    Given the operator workstation has a valid, unexpired client certificate stored in its TPM 2.0
    And a mutual TLS 1.3 session is established between the workstation and the vital processor
    When the operator sends a route-set command for route R1 over the authenticated session
    Then the vital processor shall accept and process the command
    And the route-set command shall be evaluated against the interlocking conflict matrix
    And the result shall be returned to the workstation over the same TLS session

  Scenario: TLS 1.2 connection attempt is refused
    Given the test client is configured to use TLS 1.2 maximum
    When the test client attempts to connect to the vital processor management interface
    Then the vital processor shall reject the handshake with protocol_version alert
    And the event log shall record "downgrade attempt rejected — TLS 1.2 not permitted"
```
