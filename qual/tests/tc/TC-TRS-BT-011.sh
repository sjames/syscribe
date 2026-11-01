tc_TRS_BT_011() {
    local F="$1"; local B="$F/TC-TRS-BT-011"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    # has <output> <code>   — the code is reported at all
    has() { printf '%s' "$1" | grep -q "$2"; }
    run() { "$SYSCRIBE" -m "$1" validate 2>&1 || true; }

    _scn "a known stage is accepted and an unknown one is E751"
    out=$(run "$B/valid")
    has "$out" 'E751' && fail "E751 for a valid stage: $out" || pass "bindingStage load is clean"
    out=$(run "$B/unknown")
    [ "$(cnt "$out" E751 "bogus")" -eq 1 ] && pass "E751 names the unknown stage" || fail "expected one E751: $out"

    _scn "an unstaged Configuration is checked as before"
    out=$(run "$B/unstaged")
    has "$out" 'W027' && pass "W027 still raised" || fail "W027 lost for an unstaged Configuration: $out"
    has "$out" 'E75' && fail "a staged-configuration error on an unstaged Configuration: $out" || pass "no staged-configuration finding"
}
