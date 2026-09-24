---
id: TC-TRS-LINKTYPE-011
type: TestCase
testLevel: L3
status: draft
name: "Verify custom links participate in suspect-link detection unless the type sets suspect = false."
verifies:
  - REQ-TRS-LINKTYPE-011
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-011)

  Scenario: suspect list lists unbaselined custom links except opted-out types
  Scenario: a baselined custom link raises W090 after its target changes
  Scenario: an opted-out type never raises W090
```
