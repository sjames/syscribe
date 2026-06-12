tc_TRS_SAFE_009() {
    local F="$1"; local M="$F/TC-TRS-SAFE-009/model"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" validate 2>/dev/null || true)

    SCENARIO_NAME="silLevel 4 SafetyGoal without I3 assessment raises W039"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W039 |" | grep -qF "SG-SIL4-001" \
        && pass "W039 fires for silLevel 4" || fail "W039 not fired for silLevel 4"

    SCENARIO_NAME="silLevel 3 SafetyGoal without I3 assessment raises W039"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W039 |" | grep -qF "SG-SIL3-001" \
        && pass "W039 fires for silLevel 3" || fail "W039 not fired for silLevel 3"

    SCENARIO_NAME="silLevel 2 SafetyGoal does not raise W039"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W039 |" | grep -qF "SG-SIL2-001" \
        && fail "W039 wrongly fired for silLevel 2" || pass "W039 not raised for silLevel 2"

    SCENARIO_NAME="asilLevel D still triggers W039 (regression)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W039 |" | grep -qF "SG-ASILD-001" \
        && pass "W039 fires for asilLevel D" || fail "W039 regression: asilLevel D no longer fires"

    SCENARIO_NAME="W039 message mentions IEC 61508-1"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W039 |" | grep "SG-SIL4-001" | grep -qF "61508" \
        && pass "W039 message references IEC 61508" || fail "W039 message missing IEC 61508 reference"
}
