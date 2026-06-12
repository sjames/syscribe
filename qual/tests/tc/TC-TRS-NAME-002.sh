tc_TRS_NAME_002() {
    local F="$1"; local B="$F/TC-TRS-NAME-002/model"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate 2>/dev/null || true)

    SCENARIO_NAME="E024 is retired — never emitted"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_no_code "E024"

    SCENARIO_NAME="a Requirement with id + name (no title) is clean of E024/E025"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -E "\| E02[45] \|" | grep -qF "REQ-LBL-001" \
        && fail "E024/E025 wrongly flagged the id+name Requirement" || pass "id+name Requirement clean"

    SCENARIO_NAME="a stray title on an id-identified Requirement raises E025"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    assert_has_code "E025"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E025 |" | grep -qF "REQ-LBL-002" \
        && pass "E025 names the Requirement carrying a leftover title" || fail "E025 does not name REQ-LBL-002"

    SCENARIO_NAME="a stray title on a name-identified PartDef raises E025"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E025 |" | grep -qF "BadPart" \
        && pass "E025 names the PartDef carrying a leftover title" || fail "E025 does not name BadPart"

    SCENARIO_NAME="a clean name-identified element (name only) raises no E025"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E025 |" | grep -qF "GoodPart" \
        && fail "E025 wrongly flagged a clean PartDef" || pass "clean PartDef not flagged"

    SCENARIO_NAME="a FeatureDef with a FEAT id and a name (no title) is clean of both"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -E "\| E02[45] \|" | grep -qF "AntiLock" \
        && fail "E024/E025 wrongly flagged the FeatureDef" || pass "FeatureDef id+name clean of E024/E025"
}
