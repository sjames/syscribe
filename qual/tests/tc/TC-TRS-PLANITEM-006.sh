tc_TRS_PLANITEM_006() {
    local F="$1"; local FX="$F/TC-TRS-PLANITEM-006"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)

    # 1. a leaf item marked done with resolving evidence validates cleanly
    _scn "a leaf item marked done with resolving evidence validates cleanly"
    printf '%s' "$out" | grep 'LeafDoneEvidence.md' | grep -q 'E719' \
        && fail "unexpected E719 on LeafDoneEvidence.md" || pass "LeafDoneEvidence.md raises no E719"

    # 2. a leaf item marked done with no evidence at all is rejected
    _scn "a leaf item marked done with no evidence at all is rejected"
    printf '%s' "$out" | grep 'LeafDoneNoEvidence.md' | grep -q 'E719' \
        && pass "E719 raised for a done leaf with no evidence" || fail "E719 not raised for a done leaf with no evidence"

    # 3. a leaf item marked done with only rationale-waived evidence is still rejected
    _scn "a leaf item marked done with only rationale-waived evidence is still rejected"
    printf '%s' "$out" | grep 'LeafDoneWaivedOnly.md' | grep -q 'E719' \
        && pass "E719 raised for a done leaf with only waived evidence" \
        || fail "E719 not raised for a done leaf with only waived evidence"

    # 4. a leaf item not marked done raises nothing regardless of evidence
    _scn "a leaf item not marked done raises nothing regardless of evidence"
    { printf '%s' "$out" | grep 'LeafTodo.md' | grep -q 'E719' \
      || printf '%s' "$out" | grep 'LeafInProgress.md' | grep -q 'E719' \
      || printf '%s' "$out" | grep 'LeafBlocked.md' | grep -q 'E719'; } \
        && fail "unexpected E719 on a non-done leaf" \
        || pass "todo/in_progress/blocked leaves all raise no E719"

    # 5. a non-leaf item marked done raises nothing regardless of its own evidence
    _scn "a non-leaf item marked done raises nothing regardless of its own evidence"
    printf '%s' "$out" | grep 'NonLeafDone.md' | grep -q 'E719' \
        && fail "unexpected E719 on a non-leaf" || pass "NonLeafDone.md raises no E719 (not a leaf)"
}
