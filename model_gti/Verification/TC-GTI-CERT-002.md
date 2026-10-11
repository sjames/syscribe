---
type: TestCase
id: TC-GTI-CERT-002
name: "India certification evidence"
status: active
testLevel: L5
verifies:
  - REQ-GTI-SYS-062
tags: [reg-unverified]
appliesWhen: Features::Region::IN
---

```gherkin
Feature: Certification

  Scenario: BIS registration
    Given the BIS CRS registration and test reports
    When they are inspected
    Then they cover IS 16221 Parts 1 and 2 and the IS 16169 anti-islanding test
```
