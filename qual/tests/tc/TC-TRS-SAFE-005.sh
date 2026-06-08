tc_TRS_SAFE_005() {
    local F="$1"
    local M="$F/TC-TRS-SAFE-005/metrics"

    # ── W033 gating ───────────────────────────────────────────────────────────
    run_scenario "validate metrics model — W033 on the failing goal" "$M"
    assert_no_code "E846"
    assert_has_code "W033"
    assert_output_contains "SG-MET-001"
    # SG-MET-002 passes its target, so no W033 message should name it.
    if printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W033 |" | grep -qF "SG-MET-002"; then
        fail "W033 unexpectedly names SG-MET-002"
    else
        pass "no W033 names SG-MET-002"
    fi

    # ── metrics command (text) ─────────────────────────────────────────────────
    SCENARIO_NAME="metrics text output"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" metrics 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_output_contains "SG-MET-001"
    assert_output_contains "0.945"
    if printf '%s' "$SCENARIO_OUTPUT" | grep -F "SG-MET-001" | grep -qiF "fail"; then
        pass "metrics text shows SG-MET-001 fail verdict"
    else
        fail "metrics text missing SG-MET-001 fail verdict"
    fi

    # ── metrics command (json) ──────────────────────────────────────────────────
    SCENARIO_NAME="metrics --json output"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" metrics --json 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_output_contains "\"spfm\""
    assert_output_contains "\"pmhf\""
    assert_output_contains "\"pass\""
    assert_output_contains "SG-MET-001"

    # ── E846 on out-of-range diagnosticCoverage ────────────────────────────────
    run_scenario "E846 on diagnosticCoverage out of range" "$F/TC-TRS-SAFE-005/badrange"
    assert_has_code "E846"

    # ── --deny W033 exits non-zero ──────────────────────────────────────────────
    SCENARIO_NAME="--deny W033 exits non-zero"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" validate --deny W033 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_exit_nonzero
}
