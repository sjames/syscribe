tc_TRS_PARAM_003() {
    local F="$1"; local B="$F/TC-TRS-PARAM-003"

    run_scenario "binding 99 against range 1..=8 is out of range" "$B/over"
    assert_has_code "E205"

    run_scenario "binding 8 against range 1..=8 is in range" "$B/ok"
    assert_no_code "E205"

    # GH #14 reopen: feature-check must also enforce parameter range (E205), not just validate.
    SCENARIO_NAME="feature-check enforces parameter range (E205)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/over" feature-check 2>/dev/null || true)
    assert_has_code "E205"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/ok" feature-check 2>/dev/null || true)
    assert_no_code "E205"
}
