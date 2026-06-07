tc_TRS_FMA_003() {
    local F="$1"
    local A="$F/TC-TRS-FMA-003/anomalies"
    local V="$F/TC-TRS-FMA-005/void"

    local j; j=$("$SYSCRIBE" -m "$A" feature-check --deep --json 2>/dev/null || true)

    SCENARIO_NAME="dead feature detected"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$j" | jq -e '.deadFeatures | index("Features::Dead")' >/dev/null 2>&1 \
        && pass "Dead in deadFeatures" || fail "Dead not detected"
    printf '%s' "$j" | jq -e '[.findings[].code] | index("E224")' >/dev/null 2>&1 \
        && pass "E224 emitted" || fail "E224 missing"

    SCENARIO_NAME="core feature detected"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$j" | jq -e '.coreFeatures | index("Features::Core1")' >/dev/null 2>&1 \
        && pass "Core1 in coreFeatures" || fail "Core1 not core"

    SCENARIO_NAME="false-optional detected"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$j" | jq -e '.falseOptionalFeatures | index("Features::Forced")' >/dev/null 2>&1 \
        && pass "Forced in falseOptionalFeatures" || fail "Forced not detected"
    printf '%s' "$j" | jq -e '[.findings[].code] | index("W018")' >/dev/null 2>&1 \
        && pass "W018 emitted" || fail "W018 missing"

    SCENARIO_NAME="void dominates (no dead spam)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local jv; jv=$("$SYSCRIBE" -m "$V" feature-check --deep --json 2>/dev/null || true)
    [ "$(printf '%s' "$jv" | jq -r '.void')" = "true" ] && pass "void true" || fail "void not true"
    local nd; nd=$(printf '%s' "$jv" | jq -r '.deadFeatures | length')
    [ "${nd:-0}" -eq 0 ] && pass "no per-feature dead findings when void" || fail "dead spam emitted when void (${nd})"
}
