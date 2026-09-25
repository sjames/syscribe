---
id: TC-TRS-TYPE-024
type: TestCase
testLevel: L3
status: draft
name: "Verify repoImports mounts the peer subtree at <package>::<as> and E515 detects a stable id exported by two peers."
verifies:
  - REQ-TRS-TYPE-021
---

Issue #138: resolution only worked by the peer's native qname or stable id — a reference through
the documented mount point (`Integration::Brakes::REQ-BRK-001`) was `E512` — and `E515` compared
the local model against each peer but never two peers against each other.

```gherkin
Feature: repoImports mount points and composition-wide stable ids

  Scenario: references through the mount point resolve
    Given a package mounting peer BrakeSystem as Brakes and peer Types (by trailing qname) as Lib
    And local elements that verify, derive from, satisfy, allocate to, specialize and type by peer elements through the mounts
    And a TestCase verifying the same peer requirement by peer-native qname and by stable id
    When validate runs
    Then it exits 0 with no E512, E102, E103 or E110-E114

  Scenario: a mounted reference naming nothing in the peer is E512
    Given verifies and supertype references under the mount that the peer does not define
    When validate runs
    Then E512 is raised for each

  Scenario: a stable id exported by two peers is E515
    Given two peer repos that both export REQ-BRK-001 and a local model exporting neither
    When validate runs
    Then E515 names the id and both repo aliases

  Scenario: the local-vs-peer E515 and the repo checks are unchanged
    When the TC-TRS-TYPE-021 fixtures are validated
    Then the valid composition stays clean and E515 still fires for a local-vs-peer duplicate
```
