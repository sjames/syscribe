tc_TRS_SEC_004() {
    local F="$1"

    SCENARIO_NAME="AOU.appliesTo CybersecurityGoal ref validates cleanly"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-SEC-004/model" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qE "E858|E859" \
        && fail "E858/E859 fired for valid CSG appliesTo" \
        || pass "no E858/E859 for valid CSG appliesTo"

    SCENARIO_NAME="AOU.appliesTo wrong type triggers E859"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-SEC-004/model-wrong" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "E859" \
        && pass "E859 emitted for wrong appliesTo type" \
        || fail "E859 not emitted for wrong appliesTo type"
}
