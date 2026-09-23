tc_TRS_LINKTYPE_005() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-005"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local out
    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)
    _scn "a two-element cycle in an acyclic type raises E636"
    printf '%s' "$out" | grep 'E636' | grep -q 'REQ-LT5-001' && pass "E636 for 001/002 cycle" || fail "no E636 for 001/002 cycle"
    local n; n=$(printf '%s' "$out" | grep 'E636' | grep 'REQ-LT5-00[12]' | grep -vc 'REQ-LT5-003' || true)
    [ "$n" = "1" ] && pass "cycle reported once" || fail "cycle reported $n times (expected 1)"
    _scn "a self-link in an acyclic type raises E636"
    printf '%s' "$out" | grep 'E636' | grep -q 'REQ-LT5-003' && pass "E636 for self-link" || fail "no E636 for self-link"
    _scn "a cycle in a non-acyclic type raises nothing"
    printf '%s' "$out" | grep 'E636' | grep -qE 'REQ-LT5-00[45]' && fail "E636 on non-acyclic type" || pass "non-acyclic type not checked"
    _scn "an acyclic chain raises nothing"
    printf '%s' "$out" | grep 'E636' | grep -qE 'REQ-LT5-00[67]' && fail "E636 on a chain" || pass "chain clean"
}
