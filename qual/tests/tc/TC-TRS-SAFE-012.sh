tc_TRS_SAFE_012() {
    local F="$1"
    local BASE="$F/TC-TRS-SAFE-012"

    # E865 — decomposition siblings share a satisfies target
    run_scenario "E865 — siblings share a satisfies target" "$BASE/e865_shared"
    assert_has_code "E865"

    # No E865 — distinct satisfies targets
    run_scenario "no E865 — distinct satisfies targets" "$BASE/clean"
    assert_no_code "E865"

    # W860 — single-child decomposition
    run_scenario "W860 — single-child decomposition" "$BASE/w860_single"
    assert_has_code "W860"

    # W860 draft-suppressed
    run_scenario "W860 draft-suppressed" "$BASE/w860_draft"
    assert_no_code "W860"

    # validate --deny W860 promotes to a gate failure
    _flush_scenario
    SCENARIO_NAME="validate --deny W860 promotes to gate failure"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/w860_single" validate --deny W860 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_has_code "W860"
    assert_exit_nonzero

    # decompositionKind surfaces in the safety-case report
    _flush_scenario
    SCENARIO_NAME="decompositionKind appears in safety-case report"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/safety_case" safety-case 2>/dev/null); SCENARIO_EXIT=$?
    assert_output_contains "decomposition: independent"

    # template Requirement includes a commented decompositionKind line
    _flush_scenario
    SCENARIO_NAME="template Requirement includes decompositionKind"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/safety_case" template Requirement 2>/dev/null); SCENARIO_EXIT=$?
    assert_output_contains "decompositionKind"
}
