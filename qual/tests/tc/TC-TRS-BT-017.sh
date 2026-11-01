tc_TRS_BT_017() {
    local F="$1"; local B="$F/TC-TRS-BT-017"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    has() { printf '%s' "$1" | grep -q "$2"; }
    run() { "$SYSCRIBE" -m "$1" validate 2>&1 || true; }
    res() { "$SYSCRIBE" -m "$1" residual --config "$2" --json 2>/dev/null; }

    run() { "$SYSCRIBE" -m "$1" feature-check 2>&1 || true; }  # E221/W025 are feature-check rules
    st() { res "$B/$1" CONF-BT-A1 | jq -r '.constraints[]|select(.id=="PC-BT-1")|.state'; }

    _scn "a constraint the bound values already break is violated whatever the open parameters become"
    [ "$(st viol)" = "violated" ] && pass "kv=1150, spin open: violated" || fail "expected violated, got $(st viol)"
    out=$(run "$B/viol")
    [ "$(cnt "$out" E221 "PC-BT-1")" -eq 1 ] && pass "E221 is reported for the staged Configuration" || fail "expected one E221: $out"
    printf '%s' "$out" | grep E221 | grep -q 'still-open parameters' && pass "the message names the open parameters" || fail "no open-parameter note"

    _scn "an undetermined constraint is silent"
    [ "$(st undet)" = "undetermined" ] && pass "kv=1050, spin open: undetermined" || fail "expected undetermined, got $(st undet)"
    out=$(run "$B/undet"); has "$out" 'E221' && fail "E221 for an undetermined constraint" || pass "no E221"
    [ "$(st sat)" = "satisfied" ] && pass "kv=900: satisfied for every spin" || fail "expected satisfied, got $(st sat)"

    _scn "an unstaged Configuration is judged as before"
    out=$(run "$B/unstaged_viol"); has "$out" 'E221' && fail "E221 on an unstaged, partially bound Configuration" || pass "no E221 until fully evaluable"
    out=$(run "$B/full_viol"); [ "$(cnt "$out" E221 "PC-BT-1")" -eq 1 ] && pass "fully bound violation is still E221" || fail "expected one E221: $out"

    _scn "a default that a later stage can still change is not a point"
    [ "$(st default_open)" = "undetermined" ] && pass "kv=1050 with an open dflt (default 90, window load..runtime): undetermined" || fail "expected undetermined, got $(st default_open)"
    out=$(run "$B/default_open"); has "$out" 'E221' && fail "E221 from a default that is not final: $out" || pass "no E221"

    _scn "a default is final once the window has closed"
    [ "$(st default_closed)" = "violated" ] && pass "old (window compile..load, default 90) at a runtime Configuration: violated" || fail "expected violated, got $(st default_closed)"
    [ "$(st default_still_open)" = "undetermined" ] && pass "the same constraint while its window is open: undetermined" || fail "expected undetermined, got $(st default_still_open)"

    _scn "a narrowing along the chain decides a constraint"
    [ "$(st narrow)" = "satisfied" ] && pass "kv narrowed to 900..1000: kv + 100 <= 1100 holds" || fail "expected satisfied, got $(st narrow)"
}
