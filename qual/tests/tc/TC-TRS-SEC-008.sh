tc_TRS_SEC_008() {
    local F="$1"; local M="$F/TC-TRS-SEC-008/model"

    SCENARIO_NAME="valid securityTestMethod values produce no W809"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" validate 2>/dev/null || true)
    # TC-SEC-001 (fuzz) and TC-SEC-002 (penetration_test) should not trigger W809
    w809_valid=$(printf '%s' "$out" | grep "W809" | grep -cE "TC-SEC-001|TC-SEC-002" || true)
    [ "$w809_valid" -eq 0 ] \
        && pass "no W809 for valid securityTestMethod values" \
        || fail "W809 incorrectly fired for valid securityTestMethod ($w809_valid instances)"

    SCENARIO_NAME="invalid securityTestMethod triggers W809"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qF "W809" \
        && pass "W809 emitted for invalid securityTestMethod" \
        || fail "W809 not emitted for invalid securityTestMethod"
    printf '%s' "$out" | grep -qF "unknown_method_xyz" \
        && pass "W809 names the invalid method" \
        || fail "W809 does not name the invalid method"
}
