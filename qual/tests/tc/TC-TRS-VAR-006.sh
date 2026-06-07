tc_TRS_VAR_006() {
    local F="$1"; local B="$F/TC-TRS-VAR-006"

    # ── transitive projection ────────────────────────────────────────────────
    SCENARIO_NAME="package appliesWhen gates contents transitively (--config)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local on off
    on=$("$SYSCRIBE" -m "$B/good" list Requirement --config CONF-VAR-ON-001 2>/dev/null || true)
    off=$("$SYSCRIBE" -m "$B/good" list Requirement --config CONF-VAR-OFF-001 2>/dev/null || true)
    printf '%s' "$on"  | grep -qF "Gated::GatedReq" && pass "gated req active when Variant on"  || fail "gated req missing when Variant on"
    printf '%s' "$off" | grep -qF "Gated::GatedReq" && fail "gated req wrongly active when Variant off" || pass "gated req inactive when Variant off"
    printf '%s' "$off" | grep -qF "Always::PlainReq" && pass "always-active req still present when Variant off" || fail "always-active req missing"

    SCENARIO_NAME="why-active attributes the condition to the owning package"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local wa
    wa=$("$SYSCRIBE" -m "$B/good" why-active Gated::GatedReq --config CONF-VAR-OFF-001 2>/dev/null || true)
    printf '%s' "$wa" | grep -qiF "inactive" && pass "verdict inactive in Off variant" || fail "verdict not inactive"
    printf '%s' "$wa" | grep -qiF "package" && pass "provenance names the owning package" || fail "no package provenance"
    wa=$("$SYSCRIBE" -m "$B/good" why-active Gated::GatedReq --config CONF-VAR-ON-001 2>/dev/null || true)
    printf '%s' "$wa" | grep -qiF "active" && pass "verdict active in On variant" || fail "verdict not active in On"

    SCENARIO_NAME="good model has no E228/W026"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    run_scenario "validate good" "$B/good"
    assert_no_code "E228"
    assert_no_code "W026"

    # ── E228: nesting ────────────────────────────────────────────────────────
    SCENARIO_NAME="nested appliesWhen under a gated package is E228"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    run_scenario "validate nested" "$B/nested"
    assert_has_code "E228"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E228 |" | grep -qF "BadElem" && pass "E228 names the nested element" || fail "E228 misses the nested element"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E228 |" | grep -qF "Sub"     && pass "E228 names the nested sub-package" || fail "E228 misses the nested sub-package"

    # ── E228: forbidden targets ──────────────────────────────────────────────
    SCENARIO_NAME="appliesWhen on FeatureDef / Configuration / config package is E228"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    run_scenario "validate forbidden_target" "$B/forbidden_target"
    assert_has_code "E228"

    SCENARIO_NAME="appliesWhen on the model root is E228"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    run_scenario "validate forbidden_root" "$B/forbidden_root"
    assert_has_code "E228"

    # ── W026: gates nothing ──────────────────────────────────────────────────
    SCENARIO_NAME="package gating an empty subtree is W026"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    run_scenario "validate empty" "$B/empty"
    assert_has_code "W026"
    "$SYSCRIBE" -m "$B/empty" validate --deny W026 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "--deny W026 exits non-zero" || fail "--deny W026 did not gate"

    # ── escaping references via effective condition ───────────────────────────
    SCENARIO_NAME="external ref into gated subtree escapes; internal ref does not"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/escape" validate --config CONF-VAR-OFF-001 2>/dev/null || true)
    assert_has_code "E226"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E226 |" | grep -qF "Ext"    && pass "E226 flags the external referrer" || fail "E226 does not flag Ext"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E226 |" | grep -qF "InnerB" && fail "internal sibling ref wrongly flagged" || pass "internal ref between gated siblings not flagged"
}
