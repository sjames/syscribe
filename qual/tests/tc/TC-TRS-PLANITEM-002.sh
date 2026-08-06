tc_TRS_PLANITEM_002() {
    local F="$1"; local FX="$F/TC-TRS-PLANITEM-002"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)

    # 1. a multi-level parent chain resolves correctly
    _scn "a multi-level parent chain resolves correctly"
    { printf '%s' "$out" | grep 'Top.md' | grep -qE 'E71[0-5]' \
      || printf '%s' "$out" | grep 'Mid.md' | grep -qE 'E71[0-5]' \
      || printf '%s' "$out" | grep 'Leaf.md' | grep -qE 'E71[0-5]'; } \
        && fail "unexpected hierarchy error in the 3-level chain" \
        || pass "3-level chain (Top -> Mid -> Leaf) raises no hierarchy error"

    # 2. a dangling parent is rejected
    _scn "a dangling parent is rejected"
    printf '%s' "$out" | grep 'Dangling.md' | grep -q 'E710' \
        && pass "E710 raised for a dangling parent" || fail "E710 not raised for a dangling parent"

    # 3. a wrong-type parent is rejected
    _scn "a wrong-type parent is rejected"
    printf '%s' "$out" | grep 'WrongType.md' | grep -q 'E711' \
        && pass "E711 raised for a wrong-type parent" || fail "E711 not raised for a wrong-type parent"

    # 4. a 2-node parent cycle is detected gracefully
    _scn "a 2-node parent cycle is detected gracefully"
    out=$("$SYSCRIBE" -m "$FX/cycle2" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E712' \
        && pass "E712 raised for a 2-node cycle" || fail "E712 not raised for a 2-node cycle"

    # 5. a 3-node parent cycle is detected gracefully
    _scn "a 3-node parent cycle is detected gracefully"
    out=$("$SYSCRIBE" -m "$FX/cycle3" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E712' \
        && pass "E712 raised for a 3-node cycle" || fail "E712 not raised for a 3-node cycle"
}
