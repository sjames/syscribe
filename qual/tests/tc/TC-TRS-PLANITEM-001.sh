tc_TRS_PLANITEM_001() {
    local F="$1"; local FX="$F/TC-TRS-PLANITEM-001"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)

    # 1. a valid PlanningItem validates cleanly
    _scn "a valid PlanningItem validates cleanly"
    printf '%s' "$out" | grep 'Valid.md' | grep -q 'E70' \
        && fail "unexpected PlanningItem schema error on Valid.md" \
        || pass "Valid.md raises no PlanningItem schema error"

    # 2. a malformed id is rejected
    _scn "a malformed id is rejected"
    printf '%s' "$out" | grep 'BadId.md' | grep -q 'E706' \
        && pass "E706 raised for malformed id" || fail "E706 not raised for malformed id"

    # 3. a missing required field is rejected
    _scn "a missing required field is rejected"
    printf '%s' "$out" | grep 'MissingStatus.md' | grep -q 'E707' \
        && pass "E707 raised for missing status" || fail "E707 not raised for missing status"

    # 4. an out-of-vocabulary status is rejected
    _scn "an out-of-vocabulary status is rejected"
    printf '%s' "$out" | grep 'BadStatus.md' | grep -q 'E708' \
        && pass "E708 raised for unknown status" || fail "E708 not raised for unknown status"

    # 5. an out-of-vocabulary itemType is rejected
    _scn "an out-of-vocabulary itemType is rejected"
    printf '%s' "$out" | grep 'BadItemType.md' | grep -q 'E709' \
        && pass "E709 raised for unknown itemType" || fail "E709 not raised for unknown itemType"
}
