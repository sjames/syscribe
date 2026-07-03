---
id: TC-TRS-SEARCH-003
type: TestCase
testLevel: L3
status: draft
name: "Verify clusters: TF-IDF cosine k-means grouping, deterministic init, k clamping/validation, cosine separation, config lens, and CLI/MCP parity."
verifies:
  - REQ-TRS-SEARCH-003
---

Verify that `syscribe clusters` groups elements into k topical clusters with a term label
and member ids; that every element appears in exactly one cluster; that runs are
deterministic; that a distinctive-vocabulary element separates from dissimilar ones; that
`--k` is validated/clamped; and that the MCP `clusters` tool matches `clusters --json`.

```gherkin
Feature: TF-IDF cosine k-means clusters

  Scenario: k clusters partition the elements
    When clusters --k 2 --json is invoked
    Then k is 2 and there are 2 clusters
    And every element appears in exactly one cluster
    And the cluster sizes sum to the number of clustered elements
    And each cluster carries a term label and its member ids

  Scenario: Clustering is deterministic
    When clusters --k 2 --json is invoked twice on an unchanged model
    Then both runs produce identical clusters

  Scenario: Cosine similarity groups by shared vocabulary
    Given an element with a distinctive vocabulary disjoint from the others
    When clusters --k 2 --json is invoked
    Then that element does not share a cluster with a lexically dissimilar element

  Scenario: --k is validated and clamped
    When clusters --k 0 is invoked
    Then the exit code is 1
    When clusters --k <greater than the element count> --json is invoked
    Then k is clamped to the element count

  Scenario: The MCP clusters tool matches the CLI
    Given the MCP server started on the model
    When the clusters tool is called with the same k
    Then it returns the same JSON document that clusters --json prints
    And the tool is advertised with readOnlyHint true
```
