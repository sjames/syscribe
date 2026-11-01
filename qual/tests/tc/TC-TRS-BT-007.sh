tc_TRS_BT_007() {
    local F="$1"; local B="$F/TC-TRS-BT-007"
    local out n
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }

    _scn "an unknown stage on a feature and on a parameter's latestBindingTime"
    out=$("$SYSCRIBE" -m "$B/unknown" validate 2>&1 || true)
    [ "$(cnt "$out" E230 "feature 'Features::Motor'")" -eq 1 ] && [ "$(cnt "$out" E230 "latestBindingTime 'nope'")" -eq 1 ] \
        && pass "E230 names the feature's bindingTime and the parameter's latestBindingTime" || fail "expected two E230: $(printf '%s' "$out" | grep E230)"

    _scn "a window that closes before it opens"
    out=$("$SYSCRIBE" -m "$B/inverted" validate 2>&1 || true)
    n=$(printf '%s' "$out" | grep -c 'E750' || true)
    [ "$n" -eq 2 ] && pass "E750 on the feature and on the parameter" || fail "expected two E750, got $n: $out"

    _scn "valid windows"
    out=$("$SYSCRIBE" -m "$B/valid" validate 2>&1 || true)
    printf '%s' "$out" | grep -qE 'E230|E750' && fail "finding raised for a valid window" || pass "clean"
}
