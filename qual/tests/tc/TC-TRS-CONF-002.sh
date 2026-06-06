tc_TRS_CONF_002() {
    local F="$1"; local B="$F/TC-TRS-CONF-002"

    # Scenario: template emits the canonical features: map, not selections:.
    SCENARIO_NAME="template emits features: map, not selections:"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local tpl; tpl=$("$SYSCRIBE" template Configuration 2>/dev/null || true)
    printf '%s' "$tpl" | grep -qE "^features:" \
        && pass "template contains a features: map" || fail "template missing features: map"
    printf '%s' "$tpl" | grep -qE "^selections:" \
        && fail "template still emits selections:" || pass "template has no selections: key"

    # Scenario: legacy selections: under a feature model warns (W016).
    run_scenario "legacy selections: under a feature model warns" "$B/legacy-selections"
    assert_has_code "W016"

    # Scenario: a features:-map configuration does not warn.
    run_scenario "features: map configuration does not warn" "$B/features-map"
    assert_no_code "W016"

    # Scenario: empty selections without a feature model is silent (dormant).
    run_scenario "empty selections without a feature model is silent" "$B/no-feature-model"
    assert_no_code "W016"

    # Scenario: show displays parsed feature selections.
    SCENARIO_NAME="show displays parsed feature selections"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local sh; sh=$("$SYSCRIBE" -m "$B/features-map" show CONF-CFG2-OK-001 2>/dev/null || true)
    printf '%s' "$sh" | grep -qF "Feature selections" \
        && pass "show has a Feature selections section" || fail "show missing Feature selections section"
    printf '%s' "$sh" | grep -qF "Features::Wdt" \
        && pass "show lists the selected feature" || fail "show does not list the selected feature"
}
