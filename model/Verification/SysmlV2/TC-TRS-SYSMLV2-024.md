---
id: TC-TRS-SYSMLV2-024
type: TestCase
testLevel: L3
status: active
name: "Verify an ingested SysMLv2 rendering def/rendering becomes a native RenderingDef/Rendering so a view render clause resolves to a real element."
verifies:
  - REQ-TRS-SYSMLV2-022
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_views.rs
testFunctions:
  - rendering_def_and_usage_become_real_elements
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_views.rs`; run with `cargo test -p syscribe-model --test sysmlv2_views`.

```gherkin
Feature: an ingested SysMLv2 rendering def/rendering becomes a native RenderingDef/Rendering so a view render clause resolves to a real element (TC-TRS-SYSMLV2-024)

  Scenario: rendering def and usage become native elements
    Given a rendering def and a rendering usage
    When the tool ingests the model
    Then a RenderingDef and a Rendering element exist and can be referenced by a view
```
