---
id: TC-TRS-SEARCH-001
type: TestCase
testLevel: L3
status: draft
name: "Verify ranked full-text search: BM25 relevance ordering, snippets, type/status/config scoping, empty-query and JSON contracts, and CLI/MCP parity."
verifies:
  - REQ-TRS-SEARCH-001
---

Verify that `syscribe search-text` returns elements ranked by BM25 relevance over the
Markdown body + name, best-first, each with a snippet marking the hit; that results are
ordered by descending score; that `--type`/`--status`/`--config` scope the search; that
an empty query and an unresolvable `--config` are usage errors; and that the MCP
`search_text` tool returns the same ranked document as `search-text --json`.

```gherkin
Feature: ranked full-text search

  Scenario: The most relevant element ranks first
    Given a model whose requirements mention a distinctive term
    When search-text "<that term>" is invoked
    Then the element whose body is densest in that term appears first
    And results are ordered by descending score

  Scenario: Each result carries a marked snippet
    When search-text "<term>" --json is invoked
    Then each result carries id or qname, type, score and snippet
    And the snippet marks the matched term (for example, wrapped in **)

  Scenario: --type restricts the searched set
    When search-text "<term>" --type Requirement --json is invoked
    Then every result has type Requirement

  Scenario: An empty query is a usage error
    When search-text "" is invoked
    Then the exit code is 1

  Scenario: --config searches only the variant
    Given a variant model with a requirement gated out of a configuration
    When search-text "<a term unique to the gated requirement>" --config <that config> --json is invoked
    Then the gated requirement is absent from results
    When search-text "<term>" --config bogus is invoked
    Then the exit code is 1

  Scenario: The MCP search_text tool matches the CLI
    Given the MCP server started on the model
    When the search_text tool is called with the same query
    Then it returns the same ranked JSON document that search-text --json prints
    And the tool is advertised with readOnlyHint true
```
