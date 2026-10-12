---
id: TC-TRS-VIS-032
type: TestCase
testLevel: L3
status: active
name: "Verify deterministic diagram exports: bundled text metrics, export --all and --check over the committed grid-tie renders."
verifies:
  - REQ-TRS-VIS-032
sourceFile: repo:crates/syscribe/tests/diagram_export_all.rs
testFunctions:
  - the_committed_grid_tie_renders_are_current
  - every_format_renders_the_same_bytes_twice
  - every_svg_shape_and_edge_names_its_model_element
  - editing_a_drawn_element_makes_the_check_fail_until_regenerated
  - a_render_no_diagram_produces_is_reported_then_removed
  - a_missing_render_is_reported
  - all_takes_at_most_one_configuration
tags:
  - diagrams
---

Hosted in `crates/syscribe/tests/diagram_export_all.rs` (`cargo test -p syscribe --test
diagram_export_all`); the metrics half is
`the_diagram_sizer_measures_with_the_bundled_table_not_the_installed_fonts` and
`bundled_metrics_are_liberation_sans_and_fall_back_per_character` in
`crates/syscribe-model/src/vis/metrics.rs`. CI runs `syscribe -m model_gti diagram export --all
--check`.

```gherkin
Feature: Deterministic diagram exports (TC-TRS-VIS-032)

  Scenario: same bytes
    When model_gti is exported twice in each format
    Then the two directories are identical, and the committed renders pass --check

  Scenario: drift
    When a drawn element is renamed, a render is deleted, or a diagram is removed
    Then --check exits 1 naming the file, and export --all brings the directory back in line
```
