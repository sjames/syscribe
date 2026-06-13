tc_TRS_SM_006() {
    local F="$1"
    local BASE="$F/TC-TRS-SM-006"

    # W079 fires on a transition effect that references a non-existent action
    run_scenario "W079 — dangling transition effect" "$BASE/w079_effect"
    assert_has_code "W079"

    # No W079 when entry action and effect resolve
    run_scenario "no W079 — resolvable entry action and effect" "$BASE/clean"
    assert_no_code "W079"

    # Decision transition (guarded same-source branches) is not non-determinism
    run_scenario "decision transition does not raise W072" "$BASE/decision"
    assert_no_code "W072"

    # Draft-suppressed
    run_scenario "W079 draft-suppressed" "$BASE/draft"
    assert_no_code "W079"

    # validate --deny W079 promotes to a gate failure
    _flush_scenario
    SCENARIO_NAME="validate --deny W079 promotes to gate failure"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/w079_effect" validate --deny W079 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_has_code "W079"
    assert_exit_nonzero
}
