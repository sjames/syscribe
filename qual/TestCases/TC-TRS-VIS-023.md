---
id: TC-TRS-VIS-023
type: TestCase
testLevel: L3
status: active
name: "Verify a diagram created by the New diagram dialog is written, served and editable: derived and blank requests through POST /api/elements, refusals, warnings and nested packages, with the dialog's form logic and page wiring checked by the frontend tests."
verifies:
  - REQ-TRS-VIS-023
sourceFile: repo:crates/syscribe-server/tests/new_diagram.rs
testFunctions:
  - a_derived_diagram_is_written_and_served_as_a_generated_graph
  - a_blank_diagram_is_an_empty_manifest_that_accepts_an_add
  - a_name_already_in_use_is_refused_and_nothing_is_written
  - warnings_the_new_diagram_raises_come_back_for_the_dialog
  - a_diagram_in_a_nested_package_lands_under_that_package
tags:
  - diagram
  - visualisation
  - sprotty
---

The server half is hosted in `crates/syscribe-server/tests/new_diagram.rs` (run with
`cargo test -p syscribe-server --test new_diagram`). The dialog's form logic is checked by
`crates/syscribe-server/frontend/test/new-diagram.test.mjs` and the page wiring (the dialog's ids
and the rule that a stylesheet-hidden element is shown with an explicit display) by
`test/page-wiring.test.mjs`; both run with `npm test` in `crates/syscribe-server/frontend/`.

```gherkin
Feature: creating a diagram from the browser (TC-TRS-VIS-023)

  Scenario: a derived diagram
    Given a request with diagramKind IBD and a subject and no shapes
    When it is posted to /api/elements
    Then the file holds only those two fields and the diagram model endpoint serves a generated graph flagged derived

  Scenario: a blank diagram accepts an Add
    Given a request with diagramKind BDD and shapes {}
    When it is posted and then an element is added with diagram context
    Then the diagram model is an empty manifest first and holds the new shape, pinned, afterwards

  Scenario: a name already in use
    When a diagram is posted under an existing diagram's name
    Then the response is refused with a reason and the existing file is unchanged

  Scenario: warnings return to the dialog
    When a diagram is posted whose subject has the wrong type or resolves to nothing
    Then it is written and the response carries W418 or W401

  Scenario: a nested package
    When a diagram is posted under Basics::Views
    Then it is written under that package and served

  Scenario: the form logic and the page wiring
    Then the Node tests pass for name, kind, subject and package handling and for the page's display rules
```
