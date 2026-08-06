tc_TRS_SYSMLV2_005() {
    local F="$1"; local FX="$F/TC-TRS-SYSMLV2-005"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a variant carrying @SyscribeFeature is gated like a native appliesWhen element
    _scn "a variant carrying @SyscribeFeature is gated like a native appliesWhen element"
    out=$("$SYSCRIBE" -m "$FX/model" why-active SysML2::Demo::RotorChoice::quadVariant --config CONF-QUAD-001 2>&1 || true)
    printf '%s' "$out" | grep -q 'Verdict: active' \
        && pass "quadVariant is active under CONF-QUAD-001" || fail "quadVariant not active under CONF-QUAD-001"
    out=$("$SYSCRIBE" -m "$FX/model" why-active SysML2::Demo::RotorChoice::quadVariant --config CONF-HEX-001 2>&1 || true)
    printf '%s' "$out" | grep -q 'Verdict: inactive' \
        && pass "quadVariant is inactive under CONF-HEX-001" || fail "quadVariant not inactive under CONF-HEX-001"

    # 2. feature-check --deep reports the feature model as sound
    _scn "feature-check --deep reports the feature model as sound"
    out=$("$SYSCRIBE" -m "$FX/model" feature-check --deep 2>&1 || true)
    printf '%s' "$out" | grep -q 'invalid configurations: none' \
        && pass "both Configurations are valid models" || fail "a Configuration was reported invalid"
    printf '%s' "$out" | grep -q 'void model: false' \
        && pass "feature model is not void" || fail "feature model unexpectedly void"

    # 3. a variant with no annotation stays purely structural
    _scn "a variant with no annotation stays purely structural"
    out=$("$SYSCRIBE" -m "$FX/model" why-active SysML2::Demo::RotorChoice::plainVariant --config CONF-HEX-001 2>&1 || true)
    printf '%s' "$out" | grep -q 'Verdict: always active' \
        && pass "plainVariant (no annotation) is always active, gated by nothing" \
        || fail "plainVariant unexpectedly gated"

    # 4. an unresolvable featureId is a dangling-reference finding
    _scn "an unresolvable featureId is a dangling-reference finding"
    out=$("$SYSCRIBE" -m "$FX/dangling" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E209' \
        && pass "E209 raised for the unresolvable featureId" || fail "E209 not raised"
}
