tc_TRS_SEC_005() {
    local F="$1"

    SCENARIO_NAME="CM.confirms CybersecurityGoal validates cleanly (no E851/E860)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-SEC-005/model" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qE "E851|E860" \
        && fail "E851/E860 fired for valid CSG confirms" \
        || pass "no E851/E860 for valid CSG confirms"

    SCENARIO_NAME="CM.confirms wrong element type triggers E860"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-SEC-005/model-wrong" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "E860" \
        && pass "E860 emitted for wrong confirms type" \
        || fail "E860 not emitted for wrong confirms type"
}
