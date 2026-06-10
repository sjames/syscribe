tc_TRS_ID_005() {
    local F="$1"; local B="$F/TC-TRS-ID-005"

    SCENARIO_NAME="default cap (8): 3 & 8 digit ids clean, 9 -> E023, 2 -> E006"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/default" validate 2>/dev/null || true)
    assert_has_code "E023"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E023 |" | grep -qF "REQ-IDW-000000001" \
        && pass "E023 names the 9-digit id" || fail "E023 does not name the 9-digit id"
    printf '%s' "$SCENARIO_OUTPUT" | grep -E "E023|E006" | grep -qE "REQ-IDW-001\b|REQ-IDW-00000001\b" \
        && fail "a 3- or 8-digit id was wrongly flagged" || pass "3- and 8-digit ids are clean"
    assert_has_code "E006"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E006 |" | grep -qF "REQ-IDW-01" \
        && pass "E006 flags the 2-digit id (min still 3)" || fail "2-digit id not flagged E006"

    SCENARIO_NAME="a reference to an over-long id still resolves (no E102)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_no_code "E102"

    SCENARIO_NAME="max_digits=9 widens the cap: 9-digit id is clean"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/config-wide" validate 2>/dev/null || true)
    assert_no_code "E023"

    SCENARIO_NAME="max_digits=4 tightens the cap: a 5-digit id trips E023"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/config-narrow" validate 2>/dev/null || true)
    assert_has_code "E023"
}
