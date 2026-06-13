tc_TRS_BUDGET_001() {
    local F="$1"
    local BASE="$F/TC-TRS-BUDGET-001"

    run_scenario "well-formed in-bound budget is clean" "$BASE/clean"
    for c in E866 E867 E868 W060; do assert_no_code "$c"; done

    run_scenario "E866 — evaluate not a ConstraintDef" "$BASE/e866_evaluate"
    assert_has_code "E866"

    run_scenario "E867 — malformed budget expression" "$BASE/e867_syntax"
    assert_has_code "E867"

    run_scenario "E868 — unresolved operand" "$BASE/e868_unresolved"
    assert_has_code "E868"

    run_scenario "W060 — budget violates the evaluate constraint" "$BASE/w060_violation"
    assert_has_code "W060"

    run_scenario "W060 draft-suppressed" "$BASE/draft"
    assert_no_code "W060"

    # validate --deny W060 promotes to a gate failure
    _flush_scenario
    SCENARIO_NAME="validate --deny W060 promotes to gate failure"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/w060_violation" validate --deny W060 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_has_code "W060"
    assert_exit_nonzero
}
