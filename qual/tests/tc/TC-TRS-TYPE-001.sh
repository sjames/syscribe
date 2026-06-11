tc_TRS_TYPE_001() {
    local F="$1"; local B="$F/TC-TRS-TYPE-001"

    run_scenario "declared types recognised (no E005)" "$B/ok"
    assert_no_code "E005"
    assert_exit_zero

    SCENARIO_NAME="export reports each element at its declared type"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local exp; exp=$("$SYSCRIBE" -m "$B/ok" export 2>/dev/null || true)
    printf '%s' "$exp" | grep -qF '"type": "ConstraintDef"' \
        && pass "ConstraintDef recognised in export" || fail "ConstraintDef not found at declared type in export"
    printf '%s' "$exp" | grep -qF '"type": "Constraint"' \
        && pass "Constraint recognised in export" || fail "Constraint not found at declared type in export"

    run_scenario "sibling file with bogus type produces E005" "$B/bogus"
    assert_has_code "E005"
    assert_exit_nonzero
}
