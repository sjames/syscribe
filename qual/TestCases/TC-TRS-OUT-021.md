---
id: TC-TRS-OUT-021
type: TestCase
testLevel: L3
status: draft
name: "Verify the stats corpus-shape digest: facets, coverage/orphan rollups, --group-by, scoping filters, --config lens, JSON, and CLI/MCP parity."
verifies:
  - REQ-TRS-OUT-021
---

Verify that `syscribe stats` aggregates the native `Requirement` population into
per-facet histograms (`status`, `reqDomain`, `silLevel`, `asilLevel`, `package`,
`tags`) plus coverage and orphan rollups; that `--group-by` re-keys a facet by
top-level package; that `--where`/`--status`/`--tag` scope the counted set while the
coverage rollup stays equal to the `coverage`/`matrix` numbers; that a parent
requirement is excluded from the orphan sets; that `--config` projects the digest onto
a variant; that unknown `--group-by`/`--config` are usage errors; and that the MCP
`stats` tool returns the same document as `stats --json`.

```gherkin
Feature: stats corpus-shape digest

  Scenario: The digest reports the total and every facet
    Given a model with native requirements
    When stats --json is invoked
    Then the output is valid JSON
    And it carries total, facets and coverage and orphans
    And facets carries status, reqDomain, silLevel, asilLevel, package and tags
    And silLevel and asilLevel each include a QM/none bucket for requirements declaring neither

  Scenario: Coverage equals the coverage/matrix computation
    Given a model with a mix of verified and unverified requirements
    When stats --json is invoked
    Then coverage.verified equals the verifiedCount the coverage tool reports for the same model

  Scenario: A parent requirement is excluded from the orphan sets (GH #37)
    Given a model with a parent requirement whose children derive from it
    When stats --json is invoked
    Then orphans.ids.unsatisfiedRequirements does not contain the parent id
    And orphans.ids.unverifiedRequirements does not contain the parent id
    And orphans.ids.untraced contains neither the parent nor its children

  Scenario: --group-by re-keys a facet by top-level package
    When stats --group-by status --json is invoked
    Then facets carries a byPackage map keyed by top-level package
    And each byPackage entry is a status histogram
    And the flat facets.status map is absent

  Scenario: An unknown --group-by facet is a usage error
    When stats --group-by bogus is invoked
    Then the exit code is 1
    And stderr names the valid facets

  Scenario: Scoping filters restrict the counted set but not coverage
    Given a model whose requirements have differing status values
    When stats --status approved --json is invoked
    Then total equals the number of approved requirements
    And coverage.verified is unchanged from the unfiltered run (coverage reflects the whole model)

  Scenario: --config projects the digest onto a variant
    Given a variant model with a requirement gated by appliesWhen to an unselected feature
    When stats --config <a config that excludes the feature> --json is invoked
    Then the gated requirement does not contribute to total or any facet
    When stats --config bogus is invoked
    Then the exit code is 1

  Scenario: The MCP stats tool returns the same document as the CLI
    Given the MCP server started on the model
    When the stats tool is called with no arguments
    Then it returns the same JSON document that stats --json prints
    And the tool is advertised with readOnlyHint true
```
