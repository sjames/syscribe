tc_TRS_TYPE_013() {
    local F="$1"; local B="$F/TC-TRS-TYPE-013"

    run_scenario "declared types recognised (no E005)" "$B/ok"
    assert_no_code "E005"
    assert_exit_zero

    SCENARIO_NAME="export reports each element at its declared type"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local exp; exp=$("$SYSCRIBE" -m "$B/ok" export 2>/dev/null || true)
    printf '%s' "$exp" | grep -qF '"type": "Metadata"' \
        && pass "Metadata recognised in export" || fail "Metadata not found at declared type in export"

    run_scenario "sibling file with bogus type produces E005" "$B/bogus"
    assert_has_code "E005"
    assert_exit_nonzero
}
