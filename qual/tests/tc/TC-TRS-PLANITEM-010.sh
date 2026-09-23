tc_TRS_PLANITEM_010() {
    local F="$1"; local FX="$F/TC-TRS-PLANITEM-010"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)

    # 1. done PI achieving a leaf requirement with only a draft TestCase -> W310
    _scn "a done PlanningItem achieves a leaf requirement with no active TestCase"
    local line
    line=$(printf '%s\n' "$out" | grep 'Planning/DoneLeafDraft.md' | grep 'W310' || true)
    [ -n "$line" ] && pass "W310 raised on DoneLeafDraft.md" || fail "W310 not raised on DoneLeafDraft.md"
    printf '%s' "$line" | grep -q "PI-P10-LEAFDRAFT-001" \
        && pass "W310 names the PlanningItem PI-P10-LEAFDRAFT-001" || fail "W310 does not name the PlanningItem"
    printf '%s' "$line" | grep -q "REQ-P10-LEAFDRAFT-001" \
        && pass "W310 names the Requirement REQ-P10-LEAFDRAFT-001" || fail "W310 does not name the Requirement"
    # consistency with the requirement-level bar: W002 fires on the requirement's own file
    printf '%s\n' "$out" | grep 'Requirements/REQ-P10-LEAFDRAFT-001.md' | grep -q 'W002' \
        && pass "W002 raised on the same requirement (same bar)" || fail "W002 not raised on REQ-P10-LEAFDRAFT-001.md"

    # 2. done PI achieving a leaf requirement with an active TestCase -> no W310
    _scn "a done PlanningItem achieves a leaf requirement with an active TestCase"
    printf '%s\n' "$out" | grep 'Planning/DoneLeafActive.md' | grep -q 'W310' \
        && fail "unexpected W310 on DoneLeafActive.md" || pass "no W310 on DoneLeafActive.md"
    printf '%s\n' "$out" | grep 'Requirements/REQ-P10-LEAFACTIVE-001.md' | grep -q 'W002' \
        && fail "unexpected W002 on REQ-P10-LEAFACTIVE-001.md" || pass "no W002 on the requirement either (same bar)"

    # 3. done PI achieving a parent requirement with only an active L1 TestCase -> W310
    _scn "a done PlanningItem achieves a parent requirement with only leaf-level active coverage"
    line=$(printf '%s\n' "$out" | grep 'Planning/DoneParentL1.md' | grep 'W310' || true)
    [ -n "$line" ] && pass "W310 raised on DoneParentL1.md" || fail "W310 not raised on DoneParentL1.md"
    printf '%s' "$line" | grep -q "REQ-P10-PARL1-001" \
        && pass "W310 names the parent Requirement REQ-P10-PARL1-001" || fail "W310 does not name the parent Requirement"
    printf '%s\n' "$out" | grep 'Requirements/REQ-P10-PARL1-001.md' | grep -q 'W305' \
        && pass "W305 raised on the same parent requirement (W310 matches W305's bar)" \
        || fail "W305 not raised on REQ-P10-PARL1-001.md"

    # 4. done PI achieving a parent requirement with an active L3 TestCase -> no W310
    _scn "a done PlanningItem achieves a parent requirement with active integration-level coverage"
    printf '%s\n' "$out" | grep 'Planning/DoneParentL3.md' | grep -q 'W310' \
        && fail "unexpected W310 on DoneParentL3.md" || pass "no W310 on DoneParentL3.md"
    printf '%s\n' "$out" | grep 'Requirements/REQ-P10-PARL3-001.md' | grep -q 'W305' \
        && fail "unexpected W305 on REQ-P10-PARL3-001.md" || pass "no W305 on the parent requirement either (same bar)"

    # 5. todo / in_progress / blocked items achieving an under-verified leaf -> no W310
    _scn "a non-done PlanningItem is never checked"
    local f
    for f in Todo InProgress Blocked; do
        printf '%s\n' "$out" | grep "Planning/$f.md" | grep -q 'W310' \
            && fail "unexpected W310 on $f.md" || pass "no W310 on $f.md"
    done
    # guard that the shared target really is under-verified (only a draft TestCase)
    grep -q '^status: draft' "$FX/model/Tests/TC-P10-NOTDONE-001.md" \
        && pass "fixture precondition: REQ-P10-NOTDONE-001's only TestCase is draft" \
        || fail "fixture precondition broken: TC-P10-NOTDONE-001 is not draft"

    # 6. dangling / wrong-kind achieves target -> E714/E715 only, no W310
    _scn "a dangling or wrong-kind achieves target is not re-flagged by W310"
    printf '%s\n' "$out" | grep 'Planning/DoneDangling.md' | grep -q 'E714' \
        && pass "E714 raised on DoneDangling.md" || fail "E714 not raised on DoneDangling.md"
    printf '%s\n' "$out" | grep 'Planning/DoneDangling.md' | grep -q 'W310' \
        && fail "W310 also raised on DoneDangling.md" || pass "no W310 on DoneDangling.md"
    printf '%s\n' "$out" | grep 'Planning/DoneWrongKind.md' | grep -q 'E715' \
        && pass "E715 raised on DoneWrongKind.md" || fail "E715 not raised on DoneWrongKind.md"
    printf '%s\n' "$out" | grep 'Planning/DoneWrongKind.md' | grep -q 'W310' \
        && fail "W310 also raised on DoneWrongKind.md" || pass "no W310 on DoneWrongKind.md"

    # overall: exactly the two expected W310 findings in the whole model
    local n
    n=$(printf '%s\n' "$out" | grep -c '| W310 |' || true)
    [ "$n" -eq 2 ] && pass "exactly 2 W310 findings model-wide" || fail "W310 count = $n (expected 2)"
}
