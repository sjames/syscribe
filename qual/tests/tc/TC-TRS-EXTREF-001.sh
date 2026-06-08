tc_TRS_EXTREF_001() {
    local F="$1"; local B="$F/TC-TRS-EXTREF-001"

    # Single-string and list-valued extRef parse; all unique -> no W028.
    run_scenario "single-string and list-valued extRef parse without error" "$B/clean"
    assert_exit_zero
    assert_no_code "W028"

    # Two elements sharing an extRef value -> W028.
    run_scenario "the same extRef on two elements produces W028" "$B/dup"
    assert_has_code "W028"

    # No extRef anywhere -> opt-in, no W028.
    run_scenario "a model with no extRef produces no W028" "$B/none"
    assert_no_code "W028"

    # W028 is gateable.
    SCENARIO_NAME="W028 is gateable with --deny"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B/dup" validate --deny W028 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "validate --deny W028 exits non-zero" || fail "--deny W028 did not gate"
}
