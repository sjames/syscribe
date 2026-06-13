---
id: TC-TRS-LINT-001
type: TestCase
testLevel: L3
status: draft
name: "Verify lint-docs scans external Markdown for unresolvable stable ID tokens and exits non-zero"
verifies:
  - REQ-TRS-LINT-001
---

```gherkin
Feature: lint-docs command for external documentation validation

  Scenario: resolvable stable ID token in doc produces no output and exits 0
    Given an external Markdown file that references REQ-TRS-LINT-001 which exists in the model
    When the user runs lint-docs on that file with the correct model-dir
    Then the tool produces no warning output and exits with code 0

  Scenario: unresolvable stable ID token causes W099 warning and exits 1
    Given an external Markdown file that references REQ-TRS-NONEXIST-001 which is absent from the model
    When the user runs lint-docs on that file
    Then the tool emits a W099 warning naming REQ-TRS-NONEXIST-001 and the source file and line number
    And the tool exits with a non-zero exit code

  Scenario: --json flag emits findings as JSON array
    Given an external doc with one unresolvable token REQ-TRS-NONEXIST-001
    When the user runs lint-docs --json on that file
    Then the output is a JSON array containing an entry with fields file, line, code W099, and token REQ-TRS-NONEXIST-001

  Scenario: file with no stable-ID tokens produces no output and exits 0
    Given an external Markdown file that contains no REQ-* TC-* ADR-* or other stable ID patterns
    When the user runs lint-docs on that file
    Then the tool produces no output and exits with code 0

  Scenario: directory argument recursively scans all .md files
    Given a directory containing two Markdown files one with a valid ref and one with an invalid ref
    When the user runs lint-docs on the directory path
    Then only the invalid reference causes a W099 warning
```
