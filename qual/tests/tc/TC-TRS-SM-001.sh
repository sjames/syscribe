tc_TRS_SM_001() {
    local F="$1"
    local BASE="$F/TC-TRS-SM-001"

    # Canonical nested per-substate transitions validate without legacy/deprecation noise
    run_scenario "nested per-substate transitions are canonical" "$BASE/nested"
    assert_no_code "W075"
    assert_no_code "W070"
    assert_no_code "W071"

    # Canonical top-level transitions with explicit source behave identically
    run_scenario "top-level transitions with explicit source" "$BASE/toplevel"
    assert_no_code "W075"
    assert_no_code "W070"
    assert_no_code "W071"
}
