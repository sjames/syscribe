tc_TRS_SM_005() {
    local F="$1"
    local BASE="$F/TC-TRS-SM-005"

    # Well-formed composite machine: none of the state-machine warnings
    run_scenario "well-formed composite machine is clean" "$BASE/composite_clean"
    for c in W070 W071 W072 W073 W074 W076; do assert_no_code "$c"; done

    # Recursion into an inner region with no initial
    run_scenario "W073 — inner region has no initial" "$BASE/inner_w073"
    assert_has_code "W073"

    # W076 unresolved transition endpoint
    run_scenario "W076 — unresolved transition endpoint" "$BASE/w076_unresolved"
    assert_has_code "W076"

    # Draft-suppressed
    run_scenario "W076 draft-suppressed" "$BASE/draft"
    assert_no_code "W076"

    # validate --deny W076 promotes to a gate failure
    _flush_scenario
    SCENARIO_NAME="validate --deny W076 promotes to gate failure"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/w076_unresolved" validate --deny W076 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_has_code "W076"
    assert_exit_nonzero
}
