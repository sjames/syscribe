---
id: TC-TRS-VIS-030
type: TestCase
testLevel: L3
status: active
name: "Verify external actors and the system context: actor and environment stereotypes in IBD and BDD, W996 on an unconnected boundary port, opt-in, and E992."
verifies:
  - REQ-TRS-VIS-030
sourceFile: repo:crates/syscribe-model/tests/vis_context.rs
testFunctions:
  - the_context_ibd_draws_actors_outside_the_system_and_the_environment_dashed
  - the_context_bdd_marks_actor_definitions
  - an_unconnected_system_port_in_a_context_is_w996
  - without_an_actor_there_is_no_context_and_no_w996
  - external_is_a_role_and_anything_else_is_still_e992
  - a_context_is_the_root_of_its_context_and_needs_no_user
tags:
  - diagrams
---

Hosted in `crates/syscribe-model/tests/vis_context.rs`
(`cargo test -p syscribe-model --test vis_context`), on a context holding an inverter, a PV array
and a grid as actors and a test bench as environment. The grid-tie acceptance case is
`crates/syscribe/tests/context_gti.rs` (`GtiContextIbd` with and without
`--config CONF-GTI-IN-M2-001`, and no `W996` on `model_gti`).

```gherkin
Feature: External actors and the system context (TC-TRS-VIS-030)

  Scenario: diagrams
    Then the context IBD draws the PV array and grid as actors and the bench dashed, and the BDD marks the actor definition

  Scenario: boundary check
    Then an unconnected inverter port is W996, and a composite without an actor is not checked

  Scenario: roles
    Then role external is accepted and an unknown role is E992
```
