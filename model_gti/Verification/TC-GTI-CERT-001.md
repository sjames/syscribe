---
type: TestCase
id: TC-GTI-CERT-001
name: "US certification evidence"
status: active
testLevel: L5
verifies:
  - REQ-GTI-SYS-061
tags: [reg-unverified]
appliesWhen: Features::Region::US
---

```gherkin
Feature: Certification

  Scenario: NRTL report
    Given the NRTL test report
    When it is inspected
    Then it covers UL 1741 SB with IEEE 1547.1, UL 1699B and the NEC 690 functions
```
