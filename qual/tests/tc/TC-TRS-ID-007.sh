tc_TRS_ID_007() {
    local F="$1"; local B="$F/TC-TRS-ID-007"

    SCENARIO_NAME="baseline: an unconfigured STK prefix is rejected (E006)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/baseline" validate 2>/dev/null || true)
    assert_has_code "E006"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E006 |" | grep -qF "STK-SCHED-001" \
        && pass "E006 names the unconfigured STK id" || fail "E006 does not name the STK id"

    SCENARIO_NAME="configured: STK accepted, additive, resolvable, digit-capped, per-type"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/configured" validate 2>/dev/null || true)
    printf '%s' "$SCENARIO_OUTPUT" | grep -E "E006" | grep -qF "STK-SCHED-001" \
        && fail "configured STK id was wrongly flagged E006" || pass "configured STK id STK-SCHED-001 is clean"
    printf '%s' "$SCENARIO_OUTPUT" | grep -E "E006" | grep -qF "REQ-SCHED-001" \
        && fail "built-in REQ id wrongly flagged (not additive)" || pass "built-in REQ-SCHED-001 still clean (additive)"
    assert_no_code "E102"
    pass "TestCase verifying STK-SCHED-001 resolves (no E102)"
    assert_has_code "E023"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E023 |" | grep -qF "STK-SCHED-000000001" \
        && pass "E023 applies the digit cap to the STK prefix" || fail "E023 did not flag the 9-digit STK id"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E006 |" | grep -qF "STK-TC-001" \
        && pass "E006 flags STK on a TestCase (prefix is per-type)" || fail "STK on a TestCase was not rejected"

    SCENARIO_NAME="malformed config: unknown type and bad prefix raise W046, good sibling works"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/malformed" validate 2>/dev/null || true)
    assert_has_code "W046"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W046 |" | grep -qF "Frobnicate" \
        && pass "W046 names the unknown type key" || fail "W046 does not name the unknown type key"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W046 |" | grep -qF "st-k" \
        && pass "W046 names the malformed prefix" || fail "W046 does not name the malformed prefix"
    printf '%s' "$SCENARIO_OUTPUT" | grep -E "E006" | grep -qF "GOOD-SCHED-001" \
        && fail "well-formed sibling prefix GOOD was not applied" || pass "well-formed sibling prefix GOOD takes effect"
}
