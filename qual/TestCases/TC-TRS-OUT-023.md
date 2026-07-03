---
id: TC-TRS-OUT-023
type: TestCase
testLevel: L3
status: draft
name: "Verify the hierarchical summarize digest: per-package rollup, extractive terms, representatives, content-hash cache, scope/depth/config, and CLI/MCP parity."
verifies:
  - REQ-TRS-OUT-023
---

Verify that `syscribe summarize` produces a bottom-up per-package digest — count, status
split, TF-IDF "about" terms, and representative one-liners, nested through the hierarchy;
that it caches to `.syscribe/cache/summaries.json` and yields identical output on a second
run; that `--scope`/`--depth`/`--config` behave; and that the MCP `summarize` tool returns
the same document as `summarize --json`.

```gherkin
Feature: hierarchical summarize digest

  Scenario: The digest is a nested per-package rollup
    Given a model with requirements under a package
    When summarize --json is invoked
    Then the root node carries qname, count, statusSplit, terms, representative and children
    And the count equals the number of requirements in scope
    And each child is a package node of the same shape

  Scenario: About-terms are content words, not stopwords
    When summarize --json is invoked
    Then the root terms contain no stopword (for example "the" or "shall")

  Scenario: Output is deterministic and cached
    When summarize is invoked twice on an unchanged model
    Then both runs print identical output
    And after the first run .syscribe/cache/summaries.json exists

  Scenario: --scope and --config restrict the digest
    Given a variant model with a requirement gated out of a configuration
    When summarize --json --config <that config> is invoked
    Then the root count excludes the gated requirement
    When summarize --scope <a non-existent package> is invoked
    Then the exit code is 1

  Scenario: The MCP summarize tool matches the CLI
    Given the MCP server started on the model
    When the summarize tool is called with no arguments
    Then it returns the same JSON document that summarize --json prints
    And the tool is advertised with readOnlyHint true
```
