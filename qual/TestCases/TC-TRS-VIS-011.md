---
id: TC-TRS-VIS-011
type: TestCase
testLevel: L3
status: active
name: "Verify the three layout-authority routes behind Pin all, Auto-layout and Save companion SVG: PATCH writes x/y/w/h and keeps a size on an x/y-only patch, PATCH null removes one pin and keeps the rest, DELETE drops the whole layout key and refuses non-diagrams, and PUT svg writes the companion file, sets svgMode/svgFile, appends the img once, honours an explicit svgFile and refuses a non-SVG body without writing."
verifies:
  - REQ-TRS-VIS-011
  - REQ-TRS-VIS-006
sourceFile: repo:crates/syscribe-server/tests/layout_routes.rs
testFunctions:
  - patch_with_w_and_h_writes_them
  - patch_null_removes_one_pin_and_leaves_others
  - delete_removes_the_layout_key
  - put_svg_writes_companion_sets_fields_and_appends_img_once
  - put_svg_honours_an_existing_svg_file_field
  - put_svg_refuses_non_svg_and_writes_nothing
tags:
  - diagram
  - visualisation
  - sprotty
---

Hosted integration tests in `crates/syscribe-server/tests/layout_routes.rs`; run with
`cargo test -p syscribe-server --test layout_routes`. Each test copies the shared
`tests/fixtures/mutate` model into a fresh temp directory, drives the real router in-process
(`build_router` + `new_state`) via `tower::ServiceExt::oneshot`, and re-parses the diagram's
frontmatter from disk to check what was written. The `PATCH`-`null` and `DELETE` scenarios are
the second paragraph of `REQ-TRS-VIS-006`, which this case therefore also verifies.

```gherkin
Feature: Pin all, Auto-layout and Save companion SVG are single guarded writes (TC-TRS-VIS-011)

  Scenario: a pin with a size is written and the size survives a later move
    Given the fixture diagram with one shape
    When PATCH /api/diagrams/layout carries x, y, w and h for that shape
    Then the write is committed and the layout entry on disk holds all four values
    When a later PATCH carries only x and y
    Then the new position is written and w and h are kept

  Scenario: a null value removes one pin and leaves the others
    Given a diagram with two pinned shapes
    When PATCH carries null for the first shape
    Then the write is committed, the first pin is gone and the second is unchanged
    When PATCH carries null for an already unpinned shape together with a new position for the other
    Then the write is committed as a harmless no-op for the missing pin and the other pin is updated

  Scenario: DELETE drops every pin and nothing else
    Given a diagram with two pinned shapes
    When DELETE /api/diagrams/layout is called
    Then the write is committed with a WriteResponse body, the layout key is gone, the shapes and body stay
    And GET /api/diagrams/model now reports an empty pinned set
    When DELETE is called for an unknown qname and for a PartDef
    Then each is refused with a reason saying unresolved or not a Diagram

  Scenario: PUT svg writes the companion and the fields once
    Given a diagram with no companion SVG
    When PUT /api/diagrams/svg carries a valid SVG document with surrounding whitespace
    Then the trimmed SVG is written beside the .md as <stem>.svg
    And svgMode companion and svgFile ./<stem>.svg are set, one img tag is appended and the original body is kept
    And no E402 or W405 appears in the delta
    When a second SVG is PUT
    Then the file is overwritten and the img, svgMode and svgFile each still appear once

  Scenario: an explicit svgFile is honoured
    Given a diagram declaring svgFile pictures/custom.svg
    When a valid SVG is PUT
    Then the file lands at pictures/custom.svg relative to the .md, no <stem>.svg is created, and the field is unchanged

  Scenario: a non-SVG body is refused and writes nothing
    When a div, an unterminated svg tag, an empty string and plain text are each PUT
    Then each is refused with a reason naming SVG, no file is created and the .md is byte-identical
    When a valid SVG is PUT for a PartDef
    Then it is refused as not a Diagram and no file is created
```
