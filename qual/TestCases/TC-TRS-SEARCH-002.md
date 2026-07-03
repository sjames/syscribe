---
id: TC-TRS-SEARCH-002
type: TestCase
testLevel: L3
status: draft
name: "Verify topics: per-package TF-IDF keywords, stopword demotion, type scoping, multi-package corpus, config lens, and CLI/MCP parity."
verifies:
  - REQ-TRS-SEARCH-002
---

Verify that `syscribe topics` reports per-package distinctive keywords via TF-IDF; that
stopwords are demoted; that `--type` selects the element type and yields a per-package
map; that `--config` projects; and that the MCP `topics` tool matches `topics --json`.

```gherkin
Feature: per-package TF-IDF topics

  Scenario: Each package gets a distinctive keyword list
    When topics --json is invoked
    Then the output is { packages: { <pkg>: [ {term, score} … ] } }
    And each package's terms are ordered by descending score
    And no term is a stopword (for example "the" or "shall")

  Scenario: --top bounds the term count
    When topics --top 3 --json is invoked
    Then each package lists at most 3 terms

  Scenario: --type selects the element type and spans its packages
    When topics --type FeatureDef --json is invoked
    Then the packages map has an entry per package containing a FeatureDef

  Scenario: --config projects before computing
    When topics --config bogus is invoked
    Then the exit code is 1

  Scenario: The MCP topics tool matches the CLI
    Given the MCP server started on the model
    When the topics tool is called with no arguments
    Then it returns the same JSON document that topics --json prints
    And the tool is advertised with readOnlyHint true
```
