---
id: TC-TRS-SYSMLV2-038
type: TestCase
testLevel: L3
status: active
name: "Verify the export maps every supported kind with supertype, typing, multiplicity, doc, satisfy, features and connections."
verifies:
  - REQ-TRS-SYSMLV2-038
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_export.rs
testFunctions:
  - maps_supported_kinds_with_supertype_typing_multiplicity_doc_and_satisfy
  - inline_features_and_connections_render_and_still_parse
---

```gherkin
Feature: SysML v2 export (TC-TRS-SYSMLV2-038)

  Scenario: Tool shall map supported element kinds to SysML v2 definitions and usages with supertype, typing, multiplicity, doc and satisfy
    Given a native model touching every supported kind
    When it is exported
    Then each kind appears with its supertype, typing, multiplicity, doc and satisfy, and a */ in a body cannot close the doc comment
```
