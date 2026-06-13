tc_TRS_SEC_007() {
    local F="$1"

    SCENARIO_NAME="CAL3 CSG with only I1 CM triggers W039"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-SEC-007/model-no-cm" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "W039" \
        && pass "W039 emitted for CAL3 CSG with only I1 CM" \
        || fail "W039 not emitted for CAL3 CSG with only I1 CM"

    SCENARIO_NAME="CAL3 CSG with I2 CM clears W039"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-SEC-007/model-i2" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "W039" \
        && fail "W039 fired for CAL3 CSG with I2 CM" \
        || pass "no W039 for CAL3 CSG with I2 CM"

    SCENARIO_NAME="CAL3 CSG with I3 CM also clears W039"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-SEC-007/model-i3" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "W039" \
        && fail "W039 fired for CAL3 CSG with I3 CM" \
        || pass "no W039 for CAL3 CSG with I3 CM"
}
