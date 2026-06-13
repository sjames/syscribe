tc_TRS_SM_004() {
    local F="$1"
    local BASE="$F/TC-TRS-SM-004"

    # Well-formed parallel machine: none of the state-machine warnings
    run_scenario "well-formed parallel machine is clean" "$BASE/clean"
    for c in W070 W071 W072 W073 W074 W077 W078; do assert_no_code "$c"; done

    # W073 scoped to a region with no initial
    run_scenario "W073 — region has no initial" "$BASE/w073_region"
    assert_has_code "W073"

    # W077 cross-region transition
    run_scenario "W077 — cross-region transition" "$BASE/w077_cross"
    assert_has_code "W077"

    # W078 parallel state with a single region
    run_scenario "W078 — single-region parallel state" "$BASE/w078_arity"
    assert_has_code "W078"

    # Draft-suppressed
    run_scenario "W078 draft-suppressed" "$BASE/draft"
    assert_no_code "W078"

    # validate --deny W078 promotes to a gate failure
    _flush_scenario
    SCENARIO_NAME="validate --deny W078 promotes to gate failure"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/w078_arity" validate --deny W078 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_has_code "W078"
    assert_exit_nonzero
}
