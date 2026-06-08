---
id: TC-TRS-SPEC-002
type: TestCase
testLevel: L3
status: draft
title: "Verify the discoverable spec includes a ports & interfaces decision guide."
verifies:
  - REQ-TRS-SPEC-002
---

Verify that `syscribe spec types` carries a ports/interfaces guide stating the interface-is-a-connection-of-ports relationship, the conjugation direction-flip rule, and the construct distinctions.

```gherkin
Feature: ports & interfaces decision guide is discoverable

  Scenario: spec types explains the interface/connection relationship
    When the tool runs `spec types`
    Then it states an interface is a connection whose ends are ports

  Scenario: spec types explains conjugation
    When the tool runs `spec types`
    Then it states the receiver is the conjugate (directions flip)

  Scenario: spec types distinguishes the constructs
    When the tool runs `spec types`
    Then PortDef, Port, InterfaceDef and ConnectionDef are all named in the guide
```
