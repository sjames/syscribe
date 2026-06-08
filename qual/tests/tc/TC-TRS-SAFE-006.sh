tc_TRS_SAFE_006() {
    local F="$1"
    local D="$F/TC-TRS-SAFE-006"

    # ── W034 fires on an unguarded mixed-criticality sharing ───────────────────
    run_scenario "flagged: mixed-criticality sharing with no FFI argument" "$D/flagged"
    assert_exit_zero
    assert_has_code "W034"
    assert_output_contains "SafetyCore"
    assert_output_contains "Infotainment"

    # ── ffiRationale on the target excuses the sharing ─────────────────────────
    run_scenario "excused (ffiRationale): no W034" "$D/excused"
    assert_exit_zero
    assert_no_code "W034"

    # ── accepted breakdownAdr on a source excuses the sharing ──────────────────
    run_scenario "excused (accepted breakdownAdr): no W034" "$D/excused_adr"
    assert_exit_zero
    assert_no_code "W034"

    # ── matching integrity tags → not mixed → no W034 (check still active) ──────
    run_scenario "nomix: same ASIL on both sources, no W034" "$D/nomix"
    assert_exit_zero
    assert_no_code "W034"

    # ── --deny W034 exits non-zero on the flagged model ────────────────────────
    SCENARIO_NAME="--deny W034 exits non-zero"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$D/flagged" validate --deny W034 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_exit_nonzero
}
