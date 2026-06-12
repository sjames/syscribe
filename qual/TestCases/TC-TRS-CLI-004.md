---
id: TC-TRS-CLI-004
type: TestCase
testLevel: L3
status: draft
name: "Verify model-root auto-discovery via walk-up to .syscribe.toml, flag override, and fallback."
verifies:
  - REQ-TRS-CLI-004
---

Verify that with no `-m`/`SYSCRIBE_MODEL` the tool walks up from the current directory to the nearest `.syscribe.toml` and uses that directory as the model root; that an explicit `-m` overrides discovery; and that a tree with no marker falls back to the default without spurious discovery.

```gherkin
Feature: model root resolution and auto-discovery

  Scenario: a command run from a subdirectory discovers the marked root
    Given a model whose root holds a .syscribe.toml and an element Engine
    And the current directory is a subdirectory of that model
    When the tool runs `list PartDef` with no -m and no SYSCRIBE_MODEL
    Then the element Engine is listed
    And the tool exits zero

  Scenario: an explicit -m overrides discovery
    Given the current directory is inside a marked model
    When the tool runs `-m <other-root> list PartDef`
    Then the elements of <other-root> are listed, not the discovered model's

  Scenario: explicit -m works on a model that has no .syscribe.toml
    Given a model directory with no .syscribe.toml and a current directory with no marker
    When the tool runs `-m <model> list PartDef`
    Then the model's elements are listed
    And the tool exits zero

  Scenario: no marker and no model/ falls back to the default and reports the miss
    Given a directory with no .syscribe.toml in any ancestor and no model/ subdirectory
    When the tool runs `validate` with no -m
    Then the tool does not discover a model
    And it exits non-zero reporting the missing default path
```
