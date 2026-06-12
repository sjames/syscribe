---
id: TC-TRS-VAL-012
type: TestCase
testLevel: L3
status: draft
name: "Verify sourceFile location semantics: model-relative, absolute, file URI, and remote URI."
verifies:
  - REQ-TRS-VAL-012
---

Verify that `sourceFile` values are resolved per their form — bare/`model:` (model root), absolute, and `file://` resolve and are checked locally; a remote `scheme://` URI is accepted without `W004` and skipped for `W009`; a missing local path still produces `W004`.

```gherkin
Feature: sourceFile location semantics

  Scenario: Local forms resolve and are not flagged
    Given TestCases whose sourceFile uses a bare path, a model: path, an absolute path, and a file:// URI to an existing file
    When the tool is invoked
    Then none of them produce W004 or W009

  Scenario: A missing local sourceFile is flagged
    Given a TestCase whose model: sourceFile does not exist
    When the tool is invoked
    Then exactly that TestCase produces W004

  Scenario: A remote URI sourceFile is accepted and not function-checked
    Given a TestCase whose sourceFile is an https:// URI
    When the tool is invoked
    Then it produces neither W004 nor W009
```
