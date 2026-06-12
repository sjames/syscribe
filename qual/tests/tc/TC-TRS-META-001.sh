tc_TRS_META_001() {
    local F="$1"; local M="$F/TC-TRS-META-001/model"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" validate 2>/dev/null || true)

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "an unresolved metadata application raises E317"
    assert_has_code "E317"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E317 |" | grep -qF "BadRef" \
        && pass "E317 names BadRef" || fail "E317 does not name BadRef"

    _scn "an inapplicable stereotype raises E318"
    assert_has_code "E318"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E318 |" | grep -qF "BadApply" \
        && pass "E318 names BadApply" || fail "E318 does not name BadApply"

    _scn "an undeclared tagged-value key warns W045"
    assert_has_code "W045"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W045 |" | grep -qF "BadKey" \
        && pass "W045 names BadKey" || fail "W045 does not name BadKey"

    _scn "valid applications (Pump, Valve) are clean of E317/E318/W045"
    printf '%s' "$SCENARIO_OUTPUT" | grep -E "\| (E317|E318|W045) \|" | grep -qE "Pump|Valve" \
        && fail "a valid application was wrongly flagged" || pass "Pump/Valve clean"

    _scn "show renders the applied stereotype «Critical»"
    local sh; sh=$("$SYSCRIBE" -m "$M" show System::Pump 2>/dev/null || true)
    { printf '%s' "$sh" | grep -qF "«Critical»" && printf '%s' "$sh" | grep -qF "3"; } \
        && pass "show displays «Critical» with its tagged value" || fail "show does not display the stereotype"

    _scn "list --metadata filters to elements applying the stereotype"
    local ls; ls=$("$SYSCRIBE" -m "$M" list PartDef --metadata Stereotypes::Critical 2>/dev/null || true)
    { printf '%s' "$ls" | grep -qF "Pump" && ! printf '%s' "$ls" | grep -qF "BadApply"; } \
        && pass "list --metadata includes Pump, excludes BadApply" || fail "list --metadata filter wrong"
}
