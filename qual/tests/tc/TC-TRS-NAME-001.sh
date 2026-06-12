tc_TRS_NAME_001() {
    local F="$1"; local B="$F/TC-TRS-NAME-001/model"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate 2>/dev/null || true)

    SCENARIO_NAME="a hyphenated element name raises W042"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "W042"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "Anti-Lock" \
        && pass "W042 names the hyphenated segment" || fail "W042 does not name Anti-Lock"

    SCENARIO_NAME="W042 on a FeatureDef mentions E209 consequence"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "Anti-Lock" | grep -qF "E209" 2>/dev/null; \
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep "Anti-Lock" | grep -qF "E209" \
        && pass "W042 on FeatureDef mentions E209" || fail "W042 on FeatureDef missing E209 hint"

    SCENARIO_NAME="W042 message says 'qualified-name segment' not 'name'"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "qualified-name segment" \
        && pass "W042 uses 'qualified-name segment'" || fail "W042 does not say 'qualified-name segment'"

    SCENARIO_NAME="a basic name (underscore) is not flagged"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "Traction_Control" \
        && fail "W042 wrongly flagged the underscore name" || pass "underscore name not flagged"

    SCENARIO_NAME="a stable-id-named element is exempt from W042"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "REQ-PWR-001" \
        && fail "W042 wrongly flagged a stable id" || pass "stable id exempt from W042"

    SCENARIO_NAME="an id-identified element with id+description filename is exempt from W042 (GH #44)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "AOU-KERNEL-001-AmpOutsidePerimeter" \
        && fail "W042 wrongly flagged id-prefixed filename" || pass "id+description filename exempt from W042"

    SCENARIO_NAME="a hyphenated appliesWhen reference still raises E209"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "E209"

    SCENARIO_NAME="a hyphenated directory (no _index.md) raises W042 on the directory"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "Brake-System" \
        && pass "W042 names the un-indexed hyphenated directory" || fail "W042 does not name Brake-System"

    SCENARIO_NAME="a basic directory name is not flagged"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "Powertrain" \
        && fail "W042 wrongly flagged a basic directory" || pass "basic directory not flagged"
}
