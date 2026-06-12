---
id: TC-TRS-SCRIPT-001
type: TestCase
testLevel: L3
status: draft
title: "Verify Rhai extension scripts load from the configured scripts dir, discover recursively, and support library-module import reuse."
verifies:
  - REQ-TRS-SCRIPT-001
---

Verify that the tool discovers `*.rhai` scripts under the configured `[scripts] path`
directory (recursively), that a script can `import` a reusable library module from the
scripts directory and call its functions, that scripts are not surfaced as model
elements, and that a model with no scripts directory runs normally.

```gherkin
Feature: load extension scripts from a configured directory

  Scenario: discover scripts and reuse a library module
    Given a model with a .syscribe/scripts directory containing a library and a command
    When the command is run via "scripts run"
    Then the command output reflects functions imported from the library module
    And "scripts list" enumerates the registered command

  Scenario: scripts are tooling, not model elements
    Given a model with extension scripts
    When the model elements are listed
    Then no .rhai script appears as a model element

  Scenario: a model with no scripts directory runs normally
    Given a model with no scripts directory
    When "scripts list" is run
    Then it reports that no extensions are defined and exits 0
```
