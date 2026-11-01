tc_TRS_BT_009() {
    local F="$1"; local B="$F/TC-TRS-BT-008"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }

    _scn "a mis-ordered pair is reported by both commands, once each"
    out=$("$SYSCRIBE" -m "$B/derived" validate 2>&1 || true)
    [ "$(cnt "$out" E229 "Motor.a")" -eq 1 ] && pass "validate reports it once" || fail "validate: expected one E229: $out"
    out=$("$SYSCRIBE" -m "$B/derived" feature-check 2>&1 || true)
    [ "$(cnt "$out" E229 "Motor.a")" -eq 1 ] && pass "feature-check reports it once" || fail "feature-check: expected one E229: $out"
    out=$("$SYSCRIBE" -m "$B/feature" feature-check 2>&1 || true)
    [ "$(cnt "$out" E229 "requires")" -eq 1 ] && pass "feature-check reports a feature edge once" || fail "feature-check feature edge: $out"
}
