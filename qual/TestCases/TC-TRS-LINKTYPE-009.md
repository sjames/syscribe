---
id: TC-TRS-LINKTYPE-009
type: TestCase
testLevel: L3
status: draft
name: "Verify links, refs, impact, trace and show include user-defined links."
verifies:
  - REQ-TRS-LINKTYPE-009
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-009)

  Scenario: links shows outbound and inbound custom links
  Scenario: refs shows inbound custom links
  Scenario: impact traverses custom links and filters by kind
  Scenario: trace lists custom links
  Scenario: show displays links:
```
