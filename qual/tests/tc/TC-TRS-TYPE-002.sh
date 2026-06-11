tc_TRS_TYPE_002() {
    local F="$1"; local B="$F/TC-TRS-TYPE-002"

    run_scenario "declared types recognised (no E005)" "$B/ok"
    assert_no_code "E005"
    assert_exit_zero

    SCENARIO_NAME="export reports each element at its declared type"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local exp; exp=$("$SYSCRIBE" -m "$B/ok" export 2>/dev/null || true)
    printf '%s' "$exp" | grep -qF '"type": "CalculationDef"' \
        && pass "CalculationDef recognised in export" || fail "CalculationDef not found at declared type in export"
    printf '%s' "$exp" | grep -qF '"type": "Calculation"' \
        && pass "Calculation recognised in export" || fail "Calculation not found at declared type in export"

    run_scenario "sibling file with bogus type produces E005" "$B/bogus"
    assert_has_code "E005"
    assert_exit_nonzero
}
