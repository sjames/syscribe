tc_TRS_SEC_002() {
    local F="$1"; local B="$F/TC-TRS-SEC-002"

    # W031 fires only on the untreated high/critical threat (TS-SEC-101), not on
    # the treated (TS-SEC-102) or goal-addressed (TS-SEC-103) ones; W032 fires on
    # the under-CAL goal (CSG-SEC-101). The model validates with no errors.
    run_scenario "untreated high-risk threat: W031 on it only; W032 on under-CAL goal; no errors" "$B/main"
    assert_exit_zero
    assert_has_code "W031"
    assert_count "W031" 1
    assert_has_code "W032"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W031 |" | grep -qF "TS-SEC-101" \
        && pass "W031 names the untreated TS-SEC-101" || fail "W031 not on TS-SEC-101"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W031 |" | grep -qF "TS-SEC-102" \
        && fail "W031 wrongly fired on treated TS-SEC-102" || pass "no W031 on treated TS-SEC-102"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W031 |" | grep -qF "TS-SEC-103" \
        && fail "W031 wrongly fired on addressed TS-SEC-103" || pass "no W031 on addressed TS-SEC-103"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W032 |" | grep -qF "CSG-SEC-101" \
        && pass "W032 names the under-CAL CSG-SEC-101" || fail "W032 not on CSG-SEC-101"
    # W032 message names actual vs expected CAL.
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W032 |" | grep -qF "CAL4" \
        && pass "W032 names expected CAL4" || fail "W032 missing expected CAL4"

    # Invalid riskTreatment produces E845.
    run_scenario "invalid riskTreatment produces E845" "$B/badenum"
    assert_has_code "E845"

    # W031 is gateable with --deny.
    SCENARIO_NAME="W031 is gateable with --deny"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B/main" validate --deny W031 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "validate --deny W031 exits non-zero" || fail "--deny W031 did not gate"

    # cyber-risk text shows a risk level and the untreated flag.
    SCENARIO_NAME="cyber-risk text shows risk level + untreated flag"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local t; t=$("$SYSCRIBE" -m "$B/main" cyber-risk 2>/dev/null || true)
    printf '%s' "$t" | grep -F "TS-SEC-101" | grep -qF "critical" \
        && pass "cyber-risk shows TS-SEC-101 risk = critical" || fail "cyber-risk missing TS-SEC-101 critical"
    printf '%s' "$t" | grep -F "TS-SEC-101" | grep -qF "untreated" \
        && pass "cyber-risk flags TS-SEC-101 untreated" || fail "cyber-risk missing untreated flag"
    printf '%s' "$t" | grep -F "TS-SEC-102" | grep -qF "ok" \
        && pass "cyber-risk flags treated TS-SEC-102 ok" || fail "cyber-risk TS-SEC-102 not ok"

    # cyber-risk --json is a valid JSON array carrying the documented fields.
    SCENARIO_NAME="cyber-risk --json carries the per-threat risk fields"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local j; j=$("$SYSCRIBE" -m "$B/main" cyber-risk --json 2>/dev/null || true)
    printf '%s' "$j" | jq -e 'type == "array" and length >= 3' >/dev/null 2>&1 \
        && pass "json is an array of >= 3 entries" || fail "json not an array"
    printf '%s' "$j" | jq -e '.[] | select(.id == "TS-SEC-101") | .risk == "critical" and .flag == "untreated"' >/dev/null 2>&1 \
        && pass "json TS-SEC-101 risk=critical flag=untreated" || fail "json TS-SEC-101 fields wrong"
    printf '%s' "$j" | jq -e '.[0] | has("id") and has("severity") and has("feasibility") and has("risk") and has("treatment") and has("addressed") and has("flag")' >/dev/null 2>&1 \
        && pass "json entries carry all documented fields" || fail "json entries missing fields"
}
