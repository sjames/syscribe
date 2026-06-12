---
id: TC-TRS-VAL-013
type: TestCase
testLevel: L3
status: draft
name: "Verify the remote sourceFile download hook: opt-in fetch, function verification, and retrieval-failure flagging."
verifies:
  - REQ-TRS-VAL-013
---

Verify that the `[remote]` download hook is inert without `--fetch-remote`, and that with `--fetch-remote` it fetches remote sourceFiles (enabling `W009` against the downloaded copy) and raises `W004` when retrieval fails.

```gherkin
Feature: Remote sourceFile download hook

  Scenario: The hook is inert without --fetch-remote
    Given a model with a [remote] download hook and remote sourceFiles
    When validate is invoked without --fetch-remote
    Then no W004 or W009 is emitted (the hook is not executed)

  Scenario: A fetched remote file is function-verified
    Given --fetch-remote and a hook that produces a file defining fn remote_present
    When a TestCase references a function that the fetched file does not define
    Then W009 is emitted for that function

  Scenario: A passing function in a fetched file is not flagged
    Given --fetch-remote and a fetched file defining the referenced function
    When the tool is invoked
    Then no W009 is emitted for it

  Scenario: A retrieval failure raises W004
    Given --fetch-remote and a hook that fails for a particular URL
    When a TestCase references that URL
    Then W004 reports the remote file could not be retrieved
```
