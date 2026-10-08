---
id: TC-TRS-VIS-024
type: TestCase
testLevel: L3
status: active
name: "Verify adding an existing element to a manifest diagram: written as an unpinned or pinned shape without touching the element, refused for duplicates, derived diagrams and bad targets, with the picker's search and request logic checked by the frontend tests."
verifies:
  - REQ-TRS-VIS-024
sourceFile: repo:crates/syscribe-server/tests/new_diagram.rs
testFunctions:
  - an_existing_element_is_added_as_a_pinned_shape_and_the_element_is_untouched
  - adding_an_element_already_on_the_diagram_is_refused
  - a_derived_diagram_unresolved_ref_diagram_ref_and_non_diagram_target_are_refused
  - a_shape_added_without_a_position_is_left_unpinned_for_elk
tags:
  - diagram
  - visualisation
  - sprotty
---

The server half is hosted in `crates/syscribe-server/tests/new_diagram.rs` (run with
`cargo test -p syscribe-server --test new_diagram`). The picker's search, ranking and request
logic is checked by `crates/syscribe-server/frontend/test/add-existing.test.mjs`, and the
picker's and button's ids by `test/page-wiring.test.mjs`; both run with `npm test` in
`crates/syscribe-server/frontend/`.

```gherkin
Feature: adding an existing element to a diagram (TC-TRS-VIS-024)

  Scenario: a pinned add leaves the element alone
    Given a blank manifest diagram
    When an element is posted with x and y
    Then the shape is listed with the element's kind and pinned there, the served graph shows it, and the element file is unchanged

  Scenario: an add without a position is unpinned
    When an element is posted with no x or y
    Then it is listed but no layout entry is written and the graph sends no position for it

  Scenario: a duplicate
    When an element already on the diagram is posted again
    Then the response is refused naming the existing shape and nothing is written

  Scenario: refusals
    When the target is a derived diagram, the ref is unresolved, the element is a diagram, or the target is not a diagram
    Then each is refused with a reason and nothing is written

  Scenario: the picker logic
    Then the Node tests pass for exclusion of diagrams, ranking, the result bound and the request built from a choice
```
