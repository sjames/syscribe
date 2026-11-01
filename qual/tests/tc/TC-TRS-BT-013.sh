tc_TRS_BT_013() {
    local F="$1"; local B="$F/TC-TRS-BT-013"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    # has <output> <code>   — the code is reported at all
    has() { printf '%s' "$1" | grep -q "$2"; }
    run() { "$SYSCRIBE" -m "$1" validate 2>&1 || true; }

    _scn "a derived Configuration may not bind earlier than its base"
    out=$(run "$B/earlier")
    [ "$(cnt "$out" E753 "CONF-BT-B1")" -eq 1 ] && pass "E753 names the derived Configuration" || fail "expected one E753: $out"
    out=$(run "$B/later")
    has "$out" 'E753' && fail "E753 for a later child: $out" || pass "a later child is clean"
}
