---
id: TC-TRS-SCRIPT-003
type: TestCase
testLevel: L3
status: draft
name: "Verify the read-only model API: element iteration, getters, find by id and qname, e.field, custom_fields, computed reverse indices, and print/eprint output."
verifies:
  - REQ-TRS-SCRIPT-003
---

Verify that a script can enumerate elements by type, read element getters (id, status,
title, type, tags, doc), resolve `model.find` by both id and qualified name (and unit for
unknown), read `e.field(...)` and `e.custom_fields` (unit for absent), read a computed
reverse index (`verified_by`), and write to stdout via `print` and stderr via `eprint`.

```gherkin
Feature: read-only model API for extension scripts

  Scenario: iterate and read element getters
    Given a model with requirements and a test case
    When a command iterating model.elements_of_type("Requirement") runs
    Then it prints each requirement's id, status, and reqDomain field

  Scenario: find resolves by id and by qualified name
    Given a requirement addressable by id and by qualified name
    When the command resolves both forms
    Then they refer to the same element, and an unknown reference is unit ()

  Scenario: custom fields and absent fields
    Given a requirement with custom_fields and a missing field
    When the command reads custom_fields and e.field("nope")
    Then the present custom field has its value and the missing field is unit ()

  Scenario: computed reverse index
    Given a requirement verified by a test case
    When the command reads e.verified_by
    Then it contains the verifying test case

  Scenario: stdout and stderr output
    Given a command that calls print and eprint
    When it is run
    Then the print text appears on stdout and the eprint text on stderr
```
