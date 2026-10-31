tc_TRS_BT_019() {
    local F="$1"; local B="$F/TC-TRS-BT-019"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    has() { printf '%s' "$1" | grep -q "$2"; }
    run() { "$SYSCRIBE" -m "$1" validate 2>&1 || true; }
    res() { "$SYSCRIBE" -m "$1" residual --config "$2" --json 2>/dev/null; }

    local fc
    fc() { "$SYSCRIBE" -m "$B/$1" feature-check --deep 2>&1 || true; }

    _scn "a compile Configuration that leaves no valid completion is reported, naming it"
    out=$(fc broken)
    [ "$(cnt "$out" E758 "CONF-BT-A1")" -eq 1 ] && pass "E758 names the Configuration that closed it" || fail "expected one E758 for CONF-BT-A1: $out"
    [ "$(cnt "$out" E758 "CONF-BT-B1")" -eq 0 ] && pass "the derived Configuration is not blamed" || fail "the derived Configuration was blamed too"

    _scn "a satisfiable chain is clean"
    out=$(fc fine); has "$out" 'E758\|E225\|E219' && fail "finding on a satisfiable chain: $out" || pass "clean"

    _scn "a deliberately partial Configuration is not judged complete"
    out=$("$SYSCRIBE" -m "$B/partial_ok" feature-check 2>&1 || true)
    has "$out" 'E219' && fail "E219 on a partial Configuration whose required feature is bound later: $out" || pass "no E219 for a feature left to a later stage"
    out=$(fc partial_ok); has "$out" 'E225\|E758' && fail "E225/E758 on a satisfiable partial chain: $out" || pass "no E225 or E758"

    _scn "an unrefined Configuration is judged as before"
    out=$("$SYSCRIBE" -m "$B/not_partial" feature-check 2>&1 || true)
    [ "$(cnt "$out" E219 "Features::P")" -eq 1 ] && pass "E219 when nothing refines the Configuration" || fail "expected E219: $out"
    out=$("$SYSCRIBE" -m "$B/unstaged" feature-check 2>&1 || true)
    [ "$(cnt "$out" E219 "Features::P")" -eq 1 ] && pass "E219 for an unstaged Configuration" || fail "expected E219 (unstaged): $out"
}
