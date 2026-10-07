---
id: TC-TRS-TREX-003
type: TestCase
testLevel: L3
status: active
name: "Verify trace-export --sort orders the requirement list and every nested list by directory (the walker order export uses, the default), asc or desc by qualified name, rejects an unknown value naming the valid ones, and is byte-identical across runs."
verifies:
  - REQ-TRS-TREX-003
sourceFile: repo:crates/syscribe/tests/trace_export.rs
testFunctions:
  - directory_sort_follows_the_walker_order
  - asc_and_desc_sort_every_list_by_qualified_name
  - a_bad_sort_is_a_usage_error_naming_the_valid_values
  - output_is_byte_identical_across_runs
tags:
  - traceability
  - export
  - cli
---

Black-box CLI tests in `crates/syscribe/tests/trace_export.rs`, run with
`cargo test -p syscribe --test trace_export`, on the `TC-TRS-TREX-001` model, whose parent
requirement lives one directory deeper (`Requirements/Core/`) than its children so the walker
order (`REQ-TX-001`, `-002`, `-003`, then `Core::REQ-TX-000`) differs from the qualified-name
order (`Core::REQ-TX-000` first).

```gherkin
Feature: sort orders of the trace document (TC-TRS-TREX-003)

  Scenario: directory order
    When the tool runs trace-export (with or without --sort directory)
    Then the requirements appear in the order export lists them
    And nested lists follow the same order and sort reads "directory"

  Scenario: ascending and descending
    When the tool runs trace-export --sort asc and --sort desc
    Then the requirement list, the parent's derivedChildren and a child's verifiedBy
         are ordered by full qualified name, reversed for desc, with the same entries and summary

  Scenario: an unknown order
    When the tool runs trace-export --sort random
    Then it exits 1 with nothing on stdout and names directory, asc, desc

  Scenario: determinism
    When the tool runs twice with the same options (none, --sort desc, --config CONF-TX-001)
    Then both outputs are byte-identical
```
