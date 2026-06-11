tc_TRS_NAME_002() {
    local F="$1"; local B="$F/TC-TRS-NAME-002/model"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate 2>/dev/null || true)

    SCENARIO_NAME="a name field on an id-identified type raises E024"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "E024"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E024 |" | grep -qF "REQ-LBL-001" \
        && pass "E024 names the Requirement carrying a stray name" || fail "E024 does not name REQ-LBL-001"

    SCENARIO_NAME="a clean id-identified element (id + title only) raises no E024"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E024 |" | grep -qF "REQ-LBL-002" \
        && fail "E024 wrongly flagged a clean Requirement" || pass "clean Requirement not flagged"

    SCENARIO_NAME="a title field on a name-identified type raises E025"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "E025"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E025 |" | grep -qF "BadPart" \
        && pass "E025 names the PartDef carrying a stray title" || fail "E025 does not name BadPart"

    SCENARIO_NAME="a clean name-identified element (name only) raises no E025"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E025 |" | grep -qF "GoodPart" \
        && fail "E025 wrongly flagged a clean PartDef" || pass "clean PartDef not flagged"

    SCENARIO_NAME="a FeatureDef with a FEAT id and a name (no title) is clean of both"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -E "\| E02[45] \|" | grep -qF "AntiLock" \
        && fail "E024/E025 wrongly flagged the FeatureDef" || pass "FeatureDef id+name clean of E024/E025"
}
