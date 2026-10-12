tc_TRS_FM_008() {
    local F="$1"
    local G="$F/TC-TRS-FM-008/good"
    local B="$F/TC-TRS-FM-008/bad"

    SCENARIO_NAME="dotted tree path, qname and FEAT- id all resolve in requires:"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" feature-check 2>/dev/null || true)
    assert_no_code "E212"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" validate 2>/dev/null || true)
    printf '%s' "$SCENARIO_OUTPUT" | grep -qE '^\| E[0-9]+ \|' \
        && fail "no E-level finding expected on the good fixture" \
        || pass "no E-level finding on the good fixture"

    SCENARIO_NAME="unresolved references name the accepted form and the closest match"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate 2>/dev/null || true)
    assert_has_code "E209"
    assert_output_contains "unresolved appliesWhen reference 'Features::Protection.Afci' (write the feature's qualified name"
    assert_output_contains "did you mean 'Features::Protection::Afci'?"
    assert_has_code "E239"
    assert_output_contains "did you mean 'Features::Protection::Rsd'?"
    assert_has_code "E222"
    assert_output_contains "did you mean 'Features::Mppt::GlobalScan.scanPeriodS'?"
    assert_has_code "E241"
    assert_output_contains "did you mean '{{Features::Mppt::GlobalScan.scanPeriodS}}'?"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" feature-check 2>/dev/null || true)
    assert_has_code "E212"
    assert_output_contains "dotted tree path"
}
