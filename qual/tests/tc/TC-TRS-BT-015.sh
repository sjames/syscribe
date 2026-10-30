tc_TRS_BT_015() {
    local F="$1"; local B="$F/TC-TRS-BT-015"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    # has <output> <code>   — the code is reported at all
    has() { printf '%s' "$1" | grep -q "$2"; }
    run() { "$SYSCRIBE" -m "$1" validate 2>&1 || true; }

    _scn "a required parameter whose window closed before the stage"
    out=$(run "$B/closed")
    [ "$(cnt "$out" E757 "reqearly")" -eq 1 ] && pass "E757 for the closed-window parameter" || fail "expected one E757: $out"
    [ "$(cnt "$out" W017 "reqearly")" -eq 0 ] && pass "no W017 duplicate for it" || fail "W017 raised as well as E757"

    _scn "a parameter whose window is still open is residual, not an error"
    out=$(run "$B/open")
    [ "$(cnt "$out" E757 "reqlate")" -eq 0 ] && pass "no E757 for an open window" || fail "E757 for an open window: $out"

    _scn "an unstaged Configuration is unchanged"
    out=$(run "$B/unstaged")
    has "$out" 'E757' && fail "E757 on an unstaged Configuration" || pass "no E757"
    [ "$(cnt "$out" W017 "reqearly")" -eq 1 ] && pass "W017 as before" || fail "W017 lost: $out"
}
