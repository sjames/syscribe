tc_TRS_LIB_003() {
    local F="$1"; local R="$F/TC-TRS-LIB-003/model"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$R" validate 2>/dev/null || true)

    SCENARIO_NAME="a dimension mismatch between quantity type and unit raises W044"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "W044"

    SCENARIO_NAME="W044 names the quantity type and the conflicting unit"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W044 |" | grep -qF "ISQ::MassValue" \
        && printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W044 |" | grep -qF "SI::metre" \
        && pass "W044 names ISQ::MassValue vs SI::metre" || fail "W044 does not name the mismatch"

    SCENARIO_NAME="exactly the two mismatches are flagged"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_count "W044" 2

    SCENARIO_NAME="a consistent quantity/unit pair (incl. a bare symbol) is clean"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    { printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W044 |" | grep -qF "SI::kilogram" \
        || printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W044 |" | grep -qF "SI::newton"; } \
        && fail "W044 wrongly flagged a consistent pair" || pass "consistent pairs (kilogram, newton, bare kg) clean"

    SCENARIO_NAME="an unrecognised unit makes the check lenient (no W044)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W044 |" | grep -qF "USD" \
        && fail "W044 wrongly fired on an unrecognised unit" || pass "unrecognised unit (USD) left lenient"

    SCENARIO_NAME="a non-quantity typedBy makes the check lenient (no W044)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W044 |" | grep -qF "ScalarValues::Real" \
        && fail "W044 wrongly fired on a non-quantity type" || pass "non-quantity typedBy left lenient"
}
