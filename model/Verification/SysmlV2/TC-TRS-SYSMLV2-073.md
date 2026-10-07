---
id: TC-TRS-SYSMLV2-073
type: TestCase
testLevel: L3
status: active
name: "Verify SysMLv2 ingestion and export are behaviour-identical on the current sysml-v2-parser."
verifies:
  - REQ-TRS-SYSMLV2-073
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_parser_migration.rs
testFunctions:
  - the_pinned_parser_is_zero_point_fifty_seven_or_later
  - names_resolve_against_their_own_file_when_packages_merge
  - quoted_names_and_qualified_references_are_decoded
  - doc_comments_and_literal_values_with_units_are_read
  - guard_and_assignment_expressions_render_as_before
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_parser_migration.rs`; run with `cargo test -p syscribe-model --test sysmlv2_parser_migration`. The whole `sysmlv2*` suite, the export round-trip tests and the repository ratchet are the broader regression gate.

```gherkin
Feature: Behaviour-identical SysMLv2 handling on the current parser (TC-TRS-SYSMLV2-073)

  Scenario: Names, references, literals, units, expressions and docs are read as before
    Given SysMLv2 sources that merge several files into one package
    When they are ingested
    Then every name, qualified reference, literal, unit, expression text and doc comment matches the 0.54 reading
```
