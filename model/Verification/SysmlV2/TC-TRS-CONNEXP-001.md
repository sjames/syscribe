---
id: TC-TRS-CONNEXP-001
type: TestCase
testLevel: L3
status: active
name: "Verify export-sysml keeps each connection's endpoints and export --ndjson carries source, target and ends."
verifies:
  - REQ-TRS-CONNEXP-001
sourceFile: repo:crates/syscribe/tests/sysml_export.rs
testFunctions:
  - export_sysml_pairs_each_connection_with_its_own_endpoints
  - export_ndjson_connection_record_carries_kind_and_endpoints
---

```gherkin
Feature: Connection endpoints (TC-TRS-CONNEXP-001)

  Scenario: Same-typed connections keep their endpoints through export-sysml
    Given a submodel with connections zeta, alpha and mid of one type, named out of source order
    When export-sysml runs
    Then each connection connects the ends written in the source

  Scenario: The ndjson record of a connection carries its kind and endpoints
    Given the same submodel
    When export --ndjson runs
    Then each Connection record has typedBy, source, target and ends
```
