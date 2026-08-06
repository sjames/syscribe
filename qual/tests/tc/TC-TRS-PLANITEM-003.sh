tc_TRS_PLANITEM_003() {
    local F="$1"; local FX="$F/TC-TRS-PLANITEM-003"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)

    # 1. a top-level item with a resolving achieves: validates cleanly
    _scn "a top-level item with a resolving achieves: validates cleanly"
    printf '%s' "$out" | grep 'ValidAchieves.md' | grep -qE 'E71[3-5]' \
        && fail "unexpected achieves error on ValidAchieves.md" \
        || pass "ValidAchieves.md raises no achieves error"

    # 2. a top-level item with no achieves: is rejected
    _scn "a top-level item with no achieves: is rejected"
    printf '%s' "$out" | grep 'NoAchieves.md' | grep -q 'E713' \
        && pass "E713 raised for a missing achieves" || fail "E713 not raised for a missing achieves"

    # 3. a dangling achieves: target is rejected
    _scn "a dangling achieves: target is rejected"
    printf '%s' "$out" | grep 'DanglingAchieves.md' | grep -q 'E714' \
        && pass "E714 raised for a dangling achieves target" || fail "E714 not raised for a dangling achieves target"

    # 4. a wrong-type achieves: target is rejected
    _scn "a wrong-type achieves: target is rejected"
    printf '%s' "$out" | grep 'WrongTypeAchieves.md' | grep -q 'E715' \
        && pass "E715 raised for a wrong-type achieves target" || fail "E715 not raised for a wrong-type achieves target"

    # 5. achieves: never suppresses W300
    _scn "achieves: never suppresses W300"
    printf '%s' "$out" | grep 'REQ-P3-CHILD.md' | grep -q 'W300' \
        && pass "W300 still raised for a leaf Requirement named only via achieves" \
        || fail "W300 unexpectedly suppressed for a leaf Requirement named only via achieves"

    # 6. achieves: never triggers E312
    _scn "achieves: never triggers E312"
    printf '%s' "$out" | grep -q 'E312' \
        && fail "E312 unexpectedly raised for a parent Requirement named only via achieves" \
        || pass "no E312 for a parent Requirement named only via achieves"
}
