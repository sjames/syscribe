tc_TRS_OUT_016() {
    local F="$1"
    local M="$F/TC-TRS-OUT-016/sys"

    run_n2() {
        _flush_scenario
        SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0
        printf "  ▶ %s\n" "$SCENARIO_NAME"
        shift
        SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" "$@" 2>/dev/null); SCENARIO_EXIT=$?
    }

    run_n2 "connection edges appear as named interfaces" n2 System
    assert_output_contains "IfaceAB"
    assert_output_contains "IfaceBC"

    run_n2 "--allocations adds allocation edges" n2 System --allocations
    assert_output_contains "allocatedTo"

    run_n2 "--format json matches the schema" n2 System --format json
    assert_output_contains "\"matrix\""
    assert_output_contains "\"kind\""
    assert_output_contains "IfaceAB"

    run_n2 "--format html is a table" n2 System --format html
    assert_output_contains "n2-matrix"

    run_n2 "--interfaces-only retains wired elements" n2 System --interfaces-only
    assert_output_contains "IfaceAB"
}
