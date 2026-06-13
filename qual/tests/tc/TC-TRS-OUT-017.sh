tc_TRS_OUT_017() {
    local F="$1"
    local M="$F/TC-TRS-OUT-017/imp"

    run_imp() {
        _flush_scenario
        SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0
        printf "  ▶ %s\n" "$SCENARIO_NAME"
        shift
        SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" "$@" 2>/dev/null); SCENARIO_EXIT=$?
    }

    run_imp "downstream reaches the full chain" impact SG-IMP-001
    assert_output_contains "REQ-IMP-000"
    assert_output_contains "REQ-IMP-LEAF-001"
    assert_output_contains "PartImp"
    assert_output_contains "TC-IMP-001"

    run_imp "upstream traces back to the safety goal" impact REQ-IMP-LEAF-001 --direction upstream
    assert_output_contains "SG-IMP-001"

    run_imp "--kinds restricts to verifies" impact REQ-IMP-LEAF-001 --kinds verifies
    assert_output_contains "TC-IMP-001"
    grep -q "PartImp" <<<"$SCENARIO_OUTPUT" && fail "PartImp leaked past --kinds verifies" || pass "satisfiedBy filtered out by --kinds"

    run_imp "--depth 1 limits the hops" impact SG-IMP-001 --depth 1
    assert_output_contains "REQ-IMP-000"
    grep -q "REQ-IMP-LEAF-001" <<<"$SCENARIO_OUTPUT" && fail "depth 2 leaked past --depth 1" || pass "depth limited to 1"

    run_imp "--format json schema" impact SG-IMP-001 --format json
    assert_output_contains "\"root\""
    assert_output_contains "\"nodes\""
    assert_output_contains "\"via\""

    run_imp "--format dot is a digraph" impact SG-IMP-001 --format dot
    assert_output_contains "digraph impact"

    run_imp "qualified-name root works" impact PartImp --direction upstream
    assert_output_contains "REQ-IMP-LEAF-001"
}
