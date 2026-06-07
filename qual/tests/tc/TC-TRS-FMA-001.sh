tc_TRS_FMA_001() {
    local F="$1"
    local C="$F/TC-TRS-FMA-004/cfgvalid"
    local V="$F/TC-TRS-FMA-005/void"

    # Encoding observed through the deep analysis on the shared fixtures.
    local j; j=$("$SYSCRIBE" -m "$C" feature-check --deep --json 2>/dev/null || true)

    SCENARIO_NAME="mandatory feature is core"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$j" | jq -e '.coreFeatures | index("Features::Core1")' >/dev/null 2>&1 \
        && pass "mandatory Core1 is core" || fail "mandatory Core1 not core"

    SCENARIO_NAME="alternative rejects two selected children"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$j" | jq -e '.invalidConfigurations | index("CONF-FMA4-BADALT-001")' >/dev/null 2>&1 \
        && pass "two-alternatives config invalid" || fail "alternative encoding did not reject two"

    SCENARIO_NAME="plain optional is not core"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$j" | jq -e '.coreFeatures | index("Features::ReqB") | not' >/dev/null 2>&1 \
        && pass "optional ReqB not core" || fail "optional ReqB wrongly core"

    SCENARIO_NAME="contradictory model is void"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local jv; jv=$("$SYSCRIBE" -m "$V" feature-check --deep --json 2>/dev/null || true)
    [ "$(printf '%s' "$jv" | jq -r '.void')" = "true" ] && pass "void model is void" || fail "void model not detected"
}
