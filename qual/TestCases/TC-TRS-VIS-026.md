---
id: TC-TRS-VIS-026
type: TestCase
testLevel: L3
status: active
name: "Verify the element card shown when a diagram shape or edge is clicked: identity and rendered Markdown, features resolved to their owner or type, unresolved references handled safely, with the selection logic and panel markup checked by the frontend tests."
verifies:
  - REQ-TRS-VIS-026
sourceFile: repo:crates/syscribe-server/tests/element_card.rs
testFunctions:
  - an_element_card_shows_identity_and_the_rendered_markdown_body
  - a_requirement_card_shows_its_id_and_status
  - a_port_feature_resolves_to_its_owner_and_shows_its_declared_properties
  - a_feature_without_a_resolvable_type_shows_its_owners_body
  - a_nested_feature_path_follows_typed_by_to_the_declaring_element
  - a_reference_that_names_nothing_says_so_and_is_escaped
  - an_unknown_feature_of_a_known_element_is_not_a_model_element
tags:
  - diagram
  - visualisation
  - sprotty
---

The card route is hosted in `crates/syscribe-server/tests/element_card.rs` (run with
`cargo test -p syscribe-server --test element_card`). Which element a selection means is checked
by `crates/syscribe-server/frontend/test/selection-ref.test.mjs`, and the panel's ids and its
hidden-by-attribute markup by `test/page-wiring.test.mjs`; both run with `npm test` in
`crates/syscribe-server/frontend/`.

```gherkin
Feature: reading an element from a diagram (TC-TRS-VIS-026)

  Scenario: an element card
    When the card of a documented element is requested
    Then it shows name, type, qualified name, and the body rendered with heading, table, code and a Mermaid block, plus a way to the full detail dialog

  Scenario: a requirement
    Then its id and status are shown

  Scenario: a port feature
    When the card of Sys::Controller::cmdIn is requested
    Then it names the feature and its owner, lists direction and typedBy, and shows its type's documentation, saying so

  Scenario: a feature with no resolvable type, and a nested path
    Then the owner's body is shown, and engine::powerOut follows typedBy to the element that declares powerOut

  Scenario: nothing there
    When an unknown reference, or an unknown feature of a known element, is requested
    Then the card says it is not a model element and the reference text is escaped

  Scenario: selection
    Then one selected shape or edge maps to its ref at any depth, and nothing, several, a label or a ref-less shape change nothing
```
