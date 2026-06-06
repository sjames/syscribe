tc_TRS_VAL_012() {
    local F="$1"; local SRC="$F/TC-TRS-VAL-012/src"

    # Static cases: bare + model: resolve (no W004/W009); remote URI accepted
    # (no W004/W009); only the missing model: path is flagged → exactly one W004.
    run_scenario "local forms resolve, remote accepted, missing flagged" "$SRC"
    assert_has_code "W004"
    assert_count "W004" 1
    assert_no_code "W009"

    # Absolute path and file:// URI embed machine paths, so build them at runtime
    # in a working copy pointing at that copy's tests.rs.
    local W; W=$(mktemp -d); cp -r "$SRC"/. "$W"/
    local abs="$W/tests.rs"
    _mk_tc() { # file, id, sourcefile
        cat > "$W/$1.md" <<EOF
---
id: $2
type: TestCase
title: "$1"
status: draft
testLevel: L3
verifies:
  - REQ-V12-001
sourceFile: $3
testFunctions:
  - function: "m::tests::present_case"
    scenario: "case"
---

\`\`\`gherkin
Feature: $1
  Scenario: case
    Given a source file
    Then it resolves
\`\`\`
EOF
    }
    _mk_tc tc_abs  TC-V12-ABS-001  "$abs"
    _mk_tc tc_file TC-V12-FILE-001 "file://$abs"

    SCENARIO_NAME="absolute and file:// resolve without new W004"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$W" validate 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    # Still only the one pre-existing missing path is flagged.
    assert_count "W004" 1
    assert_no_code "W009"
    rm -rf "$W"
}
