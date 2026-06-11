tc_TRS_NAME_001() {
    local F="$1"; local B="$F/TC-TRS-NAME-001/model"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate 2>/dev/null || true)

    SCENARIO_NAME="a hyphenated element name raises W042"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "W042"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "Anti-Lock" \
        && pass "W042 names the hyphenated segment" || fail "W042 does not name Anti-Lock"

    SCENARIO_NAME="a basic name (underscore) is not flagged"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "Traction_Control" \
        && fail "W042 wrongly flagged the underscore name" || pass "underscore name not flagged"

    SCENARIO_NAME="a stable-id-named element is exempt from W042"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "REQ-PWR-001" \
        && fail "W042 wrongly flagged a stable id" || pass "stable id exempt from W042"

    SCENARIO_NAME="a hyphenated appliesWhen reference still raises E209"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "E209"

    SCENARIO_NAME="a hyphenated directory (no _index.md) raises W042 on the directory"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "Brake-System" \
        && pass "W042 names the un-indexed hyphenated directory" || fail "W042 does not name Brake-System"

    SCENARIO_NAME="a basic directory name is not flagged"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W042 |" | grep -qF "Powertrain" \
        && fail "W042 wrongly flagged a basic directory" || pass "basic directory not flagged"
}
