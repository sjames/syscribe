tc_TRS_TYPE_010() {
    local F="$1"; local B="$F/TC-TRS-TYPE-010"

    run_scenario "declared types recognised (no E005)" "$B/ok"
    assert_no_code "E005"
    assert_exit_zero

    SCENARIO_NAME="export reports each element at its declared type"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local exp; exp=$("$SYSCRIBE" -m "$B/ok" export 2>/dev/null || true)
    printf '%s' "$exp" | grep -qF '"type": "SuccessionDef"' \
        && pass "SuccessionDef recognised in export" || fail "SuccessionDef not found at declared type in export"

    run_scenario "sibling file with bogus type produces E005" "$B/bogus"
    assert_has_code "E005"
    assert_exit_nonzero

    SCENARIO_NAME="show renders the declared type (GH #42 type_label)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B/ok" show OrderingDef 2>/dev/null | grep -qF "**type** | SuccessionDef |" && pass "OrderingDef shows SuccessionDef" || fail "OrderingDef not SuccessionDef (type_label regression)"
}
