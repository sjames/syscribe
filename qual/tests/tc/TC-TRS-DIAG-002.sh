tc_TRS_DIAG_002() {
    local F="$1"
    local BASE="$F/TC-TRS-DIAG-002"

    # W080 fires when a subject SendAction has no covering edge
    run_scenario "W080 — uncovered SendAction" "$BASE/missing-edge"
    assert_has_code "W080"

    # W080 is silent when every send/accept action is covered by an edge
    run_scenario "no W080 — every action covered" "$BASE/covered"
    assert_no_code "W080"

    # W080 reaches a SendAction nested inside an IfAction then-branch
    run_scenario "W080 — nested SendAction in IfAction then-branch" "$BASE/nested"
    assert_has_code "W080"

    # W080 is draft-suppressed
    run_scenario "W080 draft-suppressed" "$BASE/draft"
    assert_no_code "W080"

    # validate --deny W080 promotes the warning to a gate failure (non-zero exit)
    _flush_scenario
    SCENARIO_NAME="validate --deny W080 promotes to gate failure"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/missing-edge" validate --deny W080 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_has_code "W080"
    assert_exit_nonzero
}
