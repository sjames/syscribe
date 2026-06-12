tc_TRS_LIB_002() {
    local F="$1"; local R="$F/TC-TRS-LIB-002/model"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$R" validate 2>/dev/null || true)

    SCENARIO_NAME="a recognised ISQ type in an operation resolves cleanly (no W404)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W404 |" | grep -qF "ISQ::MassValue" \
        && fail "W404 wrongly raised for recognised ISQ::MassValue" || pass "recognised ISQ::MassValue not W404"

    SCENARIO_NAME="an unrecognised ISQ/SI member is lenient (no W043)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W043 |" | grep -qE "ISQ::|SI::" \
        && fail "W043 wrongly flagged an ISQ/SI member (open tier must be lenient)" || pass "ISQ/SI members never W043"

    SCENARIO_NAME="the closed-package typo check (LIB-001) still fires for ScalarValues"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "W043"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W043 |" | grep -qF "ScalarValues::Flota" \
        && pass "ScalarValues::Flota still raises W043" || fail "closed-package typo no longer flagged"

    SCENARIO_NAME="non-SI domain units in unit: are permissive (no finding)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$R" validate 2>/dev/null || true)
    { printf '%s' "$SCENARIO_OUTPUT" | grep -qF "USD" \
        || printf '%s' "$SCENARIO_OUTPUT" | grep -qF "kWh" \
        || printf '%s' "$SCENARIO_OUTPUT" | grep -qF "percent"; } \
        && fail "a non-SI unit was flagged" || pass "domain units (USD/kWh/percent) left permissive"
}
