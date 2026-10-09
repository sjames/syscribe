tc_TRS_FTA_005() {
    local F="$1"; local B="$F/TC-TRS-FTA-005"

    SCENARIO_NAME="OR of two events: both single point, SPFM 0.9, fails, exit 2"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/or" metrics 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_output_contains "0.9000"
    assert_output_contains "fail"
    [ "$SCENARIO_EXIT" -eq 2 ] && pass "metrics exits 2 on a failing goal" || fail "metrics exit $SCENARIO_EXIT (expected 2)"

    SCENARIO_NAME="AND of the same events: no single point, passes, exit 0"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/and" metrics 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_output_contains "1.0000"
    assert_output_contains "2.000e-10"
    assert_output_contains "pass"
    assert_exit_zero

    run_scenario "missing latentDiagnosticCoverage raises W965" "$B/missing"
    assert_has_code "W965"
    assert_output_contains "FTE-FM3-001"

    run_scenario "the passing AND model does not trip W033" "$B/and"
    assert_no_code "W033"
}
