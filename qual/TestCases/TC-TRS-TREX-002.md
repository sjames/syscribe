---
id: TC-TRS-TREX-002
type: TestCase
testLevel: L3
status: active
name: "Verify trace-export --config projects the model like export --config: inactive requirements and children are omitted, the configuration and its active features are recorded, an ad-hoc feature set is recorded without id, and an invalid configuration is a usage error."
verifies:
  - REQ-TRS-TREX-002
sourceFile: repo:crates/syscribe/tests/trace_export.rs
testFunctions:
  - config_is_null_and_nothing_filtered_without_the_lens
  - config_projection_omits_inactive_elements_and_records_the_configuration
  - an_ad_hoc_feature_set_is_recorded_without_id
  - an_invalid_configuration_is_a_usage_error
tags:
  - traceability
  - export
  - variability
  - cli
---

Black-box CLI tests in `crates/syscribe/tests/trace_export.rs`, run with
`cargo test -p syscribe --test trace_export`, on the `TC-TRS-TREX-001` model extended with the
`FeatureDef` `Features::Opt` (`FEAT-TX-OPT`), `appliesWhen: Features::Opt` on `REQ-TX-002` and
its TestCases, and `CONF-TX-001` deselecting `Opt`.

```gherkin
Feature: configuration projection of the trace document (TC-TRS-TREX-002)

  Scenario: no lens
    When the tool runs trace-export
    Then config is null and all four requirements are listed

  Scenario: a stored Configuration
    When the tool runs trace-export --config CONF-TX-001 (or its qualified name)
    Then REQ-TX-002 is absent from the list and from the parent's derivedChildren
    And config records id, qname, name and an empty activeFeatures list
    And the summary reads 3 requirements, 2 leaves, 1 satisfied, 1 verified, 1 integration-verified

  Scenario: an ad-hoc feature set
    When the tool runs trace-export --config Features::Opt (or FEAT-TX-OPT)
    Then config is {id: null, qname: null, name: "Features::Opt", activeFeatures: ["Features::Opt"]}
    And nothing is filtered

  Scenario: an invalid configuration
    When the tool runs trace-export --config CONF-NOPE-001
    Then it exits 1, prints nothing on stdout and names CONF-NOPE-001 on stderr
```
