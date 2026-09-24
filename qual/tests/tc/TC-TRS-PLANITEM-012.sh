tc_TRS_PLANITEM_012() {
    local F="$1"; local FX="$F/TC-TRS-PLANITEM-012"
    local out line n

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)

    # 1. two in_progress items sharing an achieves requirement
    _scn "two in_progress items sharing an achieves requirement raise W311"
    line=$(printf '%s\n' "$out" | grep '| W311 |' | grep 'PI-P12-ACHA-001' | grep 'PI-P12-ACHB-001' || true)
    [ -n "$line" ] && pass "W311 names both PI-P12-ACHA-001 and PI-P12-ACHB-001" \
        || fail "no W311 naming PI-P12-ACHA-001 and PI-P12-ACHB-001"
    printf '%s' "$line" | grep -q "achieves 'REQ-P12-SHARED-001'" \
        && pass "W311 names the shared Requirement REQ-P12-SHARED-001" || fail "W311 does not name the shared Requirement"
    printf '%s' "$line" | grep -q 'Planning/SharedAchA.md' \
        && pass "W311 attached to SharedAchA.md (lexically-first id)" || fail "W311 not attached to SharedAchA.md"

    # 2. two items sharing an evidence path
    _scn "two items sharing an evidence path raise W311"
    line=$(printf '%s\n' "$out" | grep '| W311 |' | grep 'PI-P12-EVA-001' | grep 'PI-P12-EVB-001' || true)
    [ -n "$line" ] && pass "W311 names both PI-P12-EVA-001 and PI-P12-EVB-001" \
        || fail "no W311 naming PI-P12-EVA-001 and PI-P12-EVB-001"
    printf '%s' "$line" | grep -q "evidence.path 'docs/shared.txt'" \
        && pass "W311 names the shared path docs/shared.txt" || fail "W311 does not name the shared path"
    printf '%s' "$line" | grep -q "achieves '" \
        && fail "evidence-path pair wrongly reported as an achieves overlap" || pass "reported as a path overlap, not achieves"

    # 3. claimed-but-todo item counts as active
    _scn "a claimed-but-todo item still counts as active"
    line=$(printf '%s\n' "$out" | grep '| W311 |' | grep 'PI-P12-CLAIMTODO-001' | grep 'PI-P12-CLAIMINP-001' || true)
    [ -n "$line" ] && pass "W311 raised for claimed-todo + in_progress pair" \
        || fail "no W311 for claimed-todo PI-P12-CLAIMTODO-001 + in_progress PI-P12-CLAIMINP-001"
    printf '%s' "$line" | grep -q "REQ-P12-CLAIM-001" \
        && pass "W311 names REQ-P12-CLAIM-001" || fail "W311 does not name REQ-P12-CLAIM-001"

    # 4. two todo, unclaimed items sharing scope
    _scn "two todo, unclaimed items sharing scope raise nothing"
    printf '%s\n' "$out" | grep '| W311 |' | grep -qE 'PI-P12-TODO[AB]-001|Planning/Todo[AB].md' \
        && fail "unexpected W311 involving TodoA/TodoB" || pass "no W311 involving the unclaimed todo pair"

    # 5. disjoint scope
    _scn "two items with disjoint scope raise nothing"
    printf '%s\n' "$out" | grep '| W311 |' | grep -qE 'PI-P12-DISJ[AB]-001|Planning/Disjoint[AB].md' \
        && fail "unexpected W311 involving DisjointA/DisjointB" || pass "no W311 involving the disjoint in_progress pair"
    n=$(printf '%s\n' "$out" | grep -c '| W311 |' || true)
    [ "$n" -eq 3 ] && pass "exactly 3 W311 findings model-wide (the three overlapping groups only)" \
        || fail "W311 count = $n (expected 3)"

    # 6. once per pair, not once per side
    _scn "W311 fires once per pair, not once per side"
    local pout
    pout=$("$SYSCRIBE" -m "$FX/pair" validate 2>&1 || true)
    n=$(printf '%s\n' "$pout" | grep -c '| W311 |' || true)
    [ "$n" -eq 1 ] && pass "exactly one W311 for the single overlapping pair" || fail "W311 count = $n (expected 1)"
    printf '%s\n' "$pout" | grep 'Planning/PairB.md' | grep -q 'W311' \
        && fail "W311 duplicated on the second side (PairB.md)" || pass "no W311 on the second side (PairB.md)"
    # in the main model too: the achieves pair is reported exactly once
    n=$(printf '%s\n' "$out" | grep '| W311 |' | grep -c 'PI-P12-ACHA-001' || true)
    [ "$n" -eq 1 ] && pass "SharedAchA/B pair reported exactly once in the main model" \
        || fail "SharedAchA/B pair reported $n times (expected 1)"
}
