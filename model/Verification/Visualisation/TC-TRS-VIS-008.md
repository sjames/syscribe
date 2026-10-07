---
id: TC-TRS-VIS-008
type: TestCase
testLevel: L2
status: active
name: "Verify the port-aware connect rules: a compatible port pair is accepted, same-direction ports are refused, a block stands in for its single compatible port and is refused when it has none or several, the connections entry is spelled as a dotted chain relative to the subject, and a derived diagram sends no manifest sync block."
verifies:
  - REQ-TRS-VIS-008
sourceFile: repo:crates/syscribe-server/frontend/test/connect-rules.test.mjs
tags:
  - diagram
  - visualisation
  - sprotty
---

A Node script, not Rust test functions: run with `npm test` from
`crates/syscribe-server/frontend/` (after `npm ci`). The rules under test are the pure
functions `resolveConnectEnds`, `compatible`, `portChain` and `isDerivedDiagram` in
`crates/syscribe-server/frontend/src/connect-rules.ts`; the script bundles that TypeScript
module with the local esbuild into `test/.build/connect-rules.mjs` (gitignored), imports it,
and drives each function over hand-built `DiagramModelSchema` fixtures — a derived IBD of
subject `A::B` (boundary with its own `mainPowerOut`, a `battery` block with one out port, a
`pdu` block with an in and an out port, all with `vis::derive`'s slug ids) and manifest
variants with author-chosen ids and port refs not spelled under the subject. No DOM, sprotty
or elkjs is involved. The script exits non-zero on the first failed assertion and prints one
`ok - <scenario>` line per scenario plus `connect-rules: ok (8 scenarios)` otherwise.

The first six scenarios are asserted directly on the rule functions. The last two are
asserted at the decision point the editor branches on: `isDerivedDiagram` is what
`src/editor.ts` consults to omit or include the `diagram:` sync block in the
`POST /api/connections` body, so the script pins its answer for slug-id, author-id, mixed
and subject-less diagrams, and checks that the resolved ends on a manifest diagram are the
port shapes whose ids the edge carries.

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
