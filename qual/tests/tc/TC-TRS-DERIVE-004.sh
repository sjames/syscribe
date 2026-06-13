tc_TRS_DERIVE_004() {
    local F="$1"

    SCENARIO_NAME="Invalid formula emits E501"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-DERIVE-004/model" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "E501" \
        && pass "E501 emitted for unparseable formula" \
        || fail "E501 not emitted for unparseable formula"

    SCENARIO_NAME="E501 message names the field 'broken'"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qF "broken" \
        && pass "E501 message contains field name 'broken'" \
        || fail "E501 message does not contain field name 'broken'"
}
