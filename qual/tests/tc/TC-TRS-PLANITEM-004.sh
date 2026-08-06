tc_TRS_PLANITEM_004() {
    local F="$1"; local FX="$F/TC-TRS-PLANITEM-004"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a gated PlanningItem is active under the selecting Configuration
    _scn "a gated PlanningItem is active under the selecting Configuration"
    out=$("$SYSCRIBE" -m "$FX/model" why-active Planning::Gated --config CONF-P4-ON 2>&1 || true)
    printf '%s' "$out" | grep -q 'Verdict: active' \
        && pass "Planning::Gated is active under CONF-P4-ON" || fail "Planning::Gated not active under CONF-P4-ON"

    # 2. a gated PlanningItem is inactive under the non-selecting Configuration
    _scn "a gated PlanningItem is inactive under the non-selecting Configuration"
    out=$("$SYSCRIBE" -m "$FX/model" why-active Planning::Gated --config CONF-P4-OFF 2>&1 || true)
    printf '%s' "$out" | grep -q 'Verdict: inactive' \
        && pass "Planning::Gated is inactive under CONF-P4-OFF" || fail "Planning::Gated not inactive under CONF-P4-OFF"

    # 3. feature-check --deep reports the feature model as sound
    _scn "feature-check --deep reports the feature model as sound"
    out=$("$SYSCRIBE" -m "$FX/model" feature-check --deep 2>&1 || true)
    printf '%s' "$out" | grep -q 'void model: false' \
        && pass "feature model is not void" || fail "feature model unexpectedly void"
    printf '%s' "$out" | grep -q 'invalid configurations: none' \
        && pass "both Configurations are valid models" || fail "a Configuration was reported invalid"
}
