tc_TRS_DERIVE_004() {
    local F="$1" out

    SCENARIO_NAME="Invalid formula emits E505"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-DERIVE-004/model" validate 2>/dev/null || true)
    grep -F "| E505 |" <<<"$out" | grep -qF "derive formula parse error" \
        && pass "E505 emitted for unparseable formula" \
        || fail "E505 not emitted for unparseable formula"
    grep -qF "| E501 |" <<<"$out" \
        && fail "E501 (the Allocation allocatedTo code) emitted for a derive parse error" \
        || pass "no E501 for a derive parse error"

    SCENARIO_NAME="E505 message names the field 'broken'"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    grep -F "| E505 |" <<<"$out" | grep -qF "broken" \
        && pass "E505 message contains field name 'broken'" \
        || fail "E505 message does not contain field name 'broken'"
}
