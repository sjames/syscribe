tc_TRS_SM_003() {
    local F="$1"
    local BASE="$F/TC-TRS-SM-003"

    run_scenario "W070 — dead state" "$BASE/w070_dead"
    assert_has_code "W070"

    run_scenario "W071 — trap state" "$BASE/w071_trap"
    assert_has_code "W071"

    run_scenario "W072 — non-determinism" "$BASE/w072_nondet"
    assert_has_code "W072"

    run_scenario "W073 — missing initial state" "$BASE/w073_noinit"
    assert_has_code "W073"

    run_scenario "W074 — multiple initial states" "$BASE/w074_multiinit"
    assert_has_code "W074"

    # Well-formed single-region machine: none of W070–W074
    run_scenario "well-formed machine is clean" "$BASE/clean"
    for c in W070 W071 W072 W073 W074; do assert_no_code "$c"; done

    # Parallel + composite machines are out of scope for the flat checks
    run_scenario "parallel/composite machines out of scope" "$BASE/outofscope"
    for c in W070 W071 W072 W073 W074; do assert_no_code "$c"; done

    # Draft-suppressed
    run_scenario "W073 draft-suppressed" "$BASE/draft"
    assert_no_code "W073"

    # validate --deny W073 promotes to a gate failure
    _flush_scenario
    SCENARIO_NAME="validate --deny W073 promotes to gate failure"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/w073_noinit" validate --deny W073 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_has_code "W073"
    assert_exit_nonzero
}
