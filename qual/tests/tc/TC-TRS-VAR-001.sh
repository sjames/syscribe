tc_TRS_VAR_001() {
    local F="$1"

    # Scenario: a flat model (no FeatureDef) emits no per-configuration findings.
    run_scenario "flat model emits no W015" "$F/TC-TRS-VAR-001/flat"
    assert_no_code "W015"

    # Scenario: matrix on a flat model falls back gracefully (notice + exit 0).
    SCENARIO_NAME="matrix on flat model falls back without error"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$F/TC-TRS-VAR-001/flat" matrix 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    [ "$SCENARIO_EXIT" -eq 0 ] && pass "matrix exits 0 when dormant" || fail "matrix exit $SCENARIO_EXIT (expected 0)"
    printf '%s' "$out" | grep -qiF "no feature model" \
        && pass "prints 'no feature model present' notice" || fail "missing dormancy notice"

    # Scenario: an unresolved appliesWhen is E209 even with no feature model.
    run_scenario "unresolved appliesWhen is E209 when dormant" "$F/TC-TRS-VAR-001/unresolved"
    assert_has_code "E209"

    # Scenario: a FeatureDef with no Configuration emits no per-config gap.
    run_scenario "feature model without Configuration emits no W015" "$F/TC-TRS-VAR-001/featuredef-no-config"
    assert_no_code "W015"
}
