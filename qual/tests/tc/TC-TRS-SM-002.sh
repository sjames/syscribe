tc_TRS_SM_002() {
    local F="$1"
    local BASE="$F/TC-TRS-SM-002"

    # W075 fires on the deprecated from/to/trigger spelling
    run_scenario "W075 — legacy from/to/trigger keys" "$BASE/legacy"
    assert_has_code "W075"

    # Canonical source/target/accept spelling is silent
    run_scenario "no W075 — canonical source/target/accept" "$BASE/canonical"
    assert_no_code "W075"

    # W075 is draft-suppressed
    run_scenario "W075 draft-suppressed" "$BASE/draft"
    assert_no_code "W075"

    # validate --deny W075 promotes the warning to a gate failure
    _flush_scenario
    SCENARIO_NAME="validate --deny W075 promotes to gate failure"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/legacy" validate --deny W075 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_has_code "W075"
    assert_exit_nonzero
}
