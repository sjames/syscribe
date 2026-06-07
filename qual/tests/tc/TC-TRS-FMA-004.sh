tc_TRS_FMA_004() {
    local F="$1"; local C="$F/TC-TRS-FMA-004/cfgvalid"
    local j; j=$("$SYSCRIBE" -m "$C" feature-check --deep --json 2>/dev/null || true)

    SCENARIO_NAME="structural violations are E225"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    for id in CONF-FMA4-BADALT-001 CONF-FMA4-BADMAND-001 CONF-FMA4-BADCHILD-001; do
        printf '%s' "$j" | jq -e --arg id "$id" '.invalidConfigurations | index($id)' >/dev/null 2>&1 \
            && pass "$id invalid (E225)" || fail "$id not flagged E225"
    done

    SCENARIO_NAME="valid configuration not flagged"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$j" | jq -e '.invalidConfigurations | index("CONF-FMA4-GOOD-001") | not' >/dev/null 2>&1 \
        && pass "GOOD config not flagged" || fail "GOOD config wrongly flagged"

    SCENARIO_NAME="requires violation stays E219, not E225"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$j" | jq -e '[.findings[].code] | index("E219")' >/dev/null 2>&1 \
        && pass "E219 emitted for requires violation" || fail "E219 missing"
    printf '%s' "$j" | jq -e '.invalidConfigurations | index("CONF-FMA4-REQVIOL-001") | not' >/dev/null 2>&1 \
        && pass "requires-violation not duplicated as E225" || fail "requires violation wrongly in E225"
}
