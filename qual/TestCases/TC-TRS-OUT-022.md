---
id: TC-TRS-OUT-022
type: TestCase
testLevel: L3
status: approved
name: "Verify the digest bulk view: compact NDJSON rows, one-line text, paging, scoping filters, --config lens, JSON document, and CLI/MCP parity."
verifies:
  - REQ-TRS-OUT-022
---

Verify that `syscribe digest` emits one compact line per native `Requirement` carrying
`id`/`name`/`status`/`reqDomain`/`text`/`verified`; that `text` is a single bounded
line; that `--limit`/`--offset` page the rows while `total` stays the full in-scope
count; that `--status`/`--tag`/`--where` and `--config` scope the set; and that the MCP
`digest` tool returns the same rows as `digest --json`.

```gherkin
Feature: digest bulk requirement view

  Scenario: Default output is one compact NDJSON row per requirement
    Given a model with native requirements
    When digest is invoked
    Then each line is a valid JSON object
    And each object carries id, name, status, reqDomain, text and verified
    And the text field contains no embedded newline and is length-bounded

  Scenario: --json emits a paged document with a pre-paging total
    Given a model with more than five requirements
    When digest --json --limit 3 --offset 2 is invoked
    Then the output is one document with total, offset and rows
    And rows has at most 3 entries
    And total equals the full in-scope requirement count (not the paged count)

  Scenario: Scoping filters restrict the rows and the total
    When digest --json --status approved is invoked
    Then total equals the number of approved requirements
    And every row has status approved

  Scenario: --config projects the rows onto a variant
    Given a variant model with a requirement gated by appliesWhen to an unselected feature
    When digest --json --config <a config that excludes the feature> is invoked
    Then the gated requirement id is absent from rows
    When digest --config bogus is invoked
    Then the exit code is 1

  Scenario: The MCP digest tool returns the same document as the CLI
    Given the MCP server started on the model
    When the digest tool is called with no arguments
    Then it returns the same JSON document that digest --json prints
    And the tool is advertised with readOnlyHint true
```
