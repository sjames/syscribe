---
id: TC-TRS-HPLE-007
type: TestCase
testLevel: L4
status: active
name: "Verify a Configuration consolidates already-configured lower-tier product-line models end to end: local and peer-repo subConfigurations, transitive multi-tier chains, and the real walk-and-validate gate."
sourceFile: repo:crates/syscribe-model/tests/hple_subconfigurations.rs
testFunctions:
  - peer_valid_configuration_validates_cleanly
  - peer_invalid_configuration_is_caught_via_real_walk_and_validate
  - three_level_local_chain_propagates_transitively
  - deep_acyclic_subconfigurations_chain_hits_the_bounded_depth_guard_without_crashing
  - cross_repo_cyclic_subconfigurations_reports_e518_even_on_a_constrained_stack
verifies:
  - REQ-TRS-HPLE-000
tags:
  - variability
  - multi-repo
---

System-level integration case for the hierarchical product-line stakeholder requirement: a
higher-tier `Configuration` consolidates lower-tier product lines that live either locally or in a
`[repos]`-mounted peer repository, at any depth. Hosted in
`crates/syscribe-model/tests/hple_subconfigurations.rs`; run with
`cargo test -p syscribe-model --test hple_subconfigurations`.

```gherkin
Feature: Consolidating configured lower-tier product lines (TC-TRS-HPLE-007)

  Scenario: A peer repository's configured product line consolidates cleanly
    Given a higher-tier Configuration whose subConfigurations names a Configuration in a [repos]-mounted peer
    When the higher-tier model is validated
    Then the peer Configuration is loaded and validated for real and no error is raised

  Scenario: An invalid lower tier is caught by the real walk-and-validate gate
    Given the peer Configuration is internally invalid
    When the higher-tier model is validated
    Then E518 names the sub-configuration and the higher tier does not claim it as consolidated

  Scenario: Consolidation is transitive across tiers
    Given a three-level local chain of Configurations and a deep acyclic chain beyond the depth guard
    When the top-level model is validated
    Then parameters and validity propagate through every tier and the depth guard terminates without crashing

  Scenario: A cross-repository cycle is reported, never looped
    Given two repositories whose Configurations consolidate each other
    When either is validated
    Then E518 is reported even on a constrained stack
```
