tc_TRS_CFLD_001() {
    local F="$1"; local B="$F/TC-TRS-CFLD-001"

    SCENARIO_NAME="scalar and list-of-scalar custom fields validate clean"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/valid" validate 2>/dev/null || true)
    assert_no_code "W041"

    SCENARIO_NAME="a nested-map custom field raises W041"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/bad" validate 2>/dev/null || true)
    assert_has_code "W041"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W041 |" | grep -qF "meta" \
        && pass "W041 names the offending key" || fail "W041 does not name 'meta'"
}
