---
id: TC-TRS-VIS-008
type: TestCase
testLevel: L2
status: draft
name: "Verify the port-aware connect rules: a compatible port pair is accepted, same-direction ports are refused, a block stands in for its single compatible port and is refused when it has none or several, the connections entry is spelled as a dotted chain relative to the subject, and a derived diagram sends no manifest sync block."
verifies:
  - REQ-TRS-VIS-008
sourceFile: repo:crates/syscribe-server/frontend/src/connect-rules.ts
tags:
  - diagram
  - visualisation
  - sprotty
---

**Draft.** The rules under test are the pure functions `resolveConnectEnds`, `compatible`,
`portChain` and `isDerivedDiagram` in `crates/syscribe-server/frontend/src/connect-rules.ts`;
they have no automated test yet. This case becomes `active` when a Node script under
`crates/syscribe-server/frontend/test/` (run by `npm test`) exercises them over fixture schemas,
at which point `sourceFile` moves to that script. Until then the scenarios below are the
specification the script must satisfy; they were checked by hand against the source on
2026-10-07.

```gherkin
Feature: the connect gesture is port-aware (TC-TRS-VIS-008)

  Scenario: port to port with compatible directions
    Given an out port on the battery block and an in port on the pdu block
    When the user clicks the out port then the in port in Connect mode
    Then the two ports are the ends of the new connection edge

  Scenario: a block stands in for its single compatible port
    Given a battery block with exactly one out port and a pdu block with one in port and one out port
    When the user clicks the battery block then the pdu block
    Then the battery's out port and the pdu's in port are the ends, because that is the only compatible pair

  Scenario: two ports of the same direction are refused
    Given two out ports
    When the user connects one to the other
    Then the gesture is refused with a toast naming both directions and nothing is written

  Scenario: an ambiguous block pair is refused
    Given a source block with two out ports and a target block with two in ports
    When the user clicks one block then the other
    Then the gesture is refused as ambiguous, the toast lists every compatible pair and asks for the two ports to be connected directly

  Scenario: a block with no ports is refused
    Given a block that owns no port children
    When the user clicks it as either end
    Then the gesture is refused with a toast saying it has no ports to connect from or to

  Scenario: the connections entry is a dotted chain relative to the subject
    Given an IBD whose subject is A::B, a port A::B::battery::powerOut under the battery block and the subject's own port A::B::mainPowerOut
    When an accepted gesture is sent to POST /api/connections
    Then qname is A::B, from is battery.powerOut and to is mainPowerOut
    And a manifest port whose ref is not spelled under the subject is rebuilt from the enclosing blocks' names, skipping the boundary

  Scenario: a derived diagram sends no diagram sync block
    Given a diagram with a subject whose every root shape id is the slug of its ref
    When an accepted gesture is sent
    Then the request carries no diagram block, and the connections entry is the whole transaction

  Scenario: a manifest diagram syncs the edge
    Given a diagram whose shapes carry author-chosen ids
    When an accepted gesture is sent
    Then the request carries a diagram block with the edge id and both port shape ids
```
