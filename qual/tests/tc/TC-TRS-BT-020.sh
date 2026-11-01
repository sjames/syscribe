tc_TRS_BT_020() {
    local F="$1"; local B="$F/TC-TRS-BT-020"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    has() { printf '%s' "$1" | grep -q "$2"; }
    run() { "$SYSCRIBE" -m "$1" validate 2>&1 || true; }
    res() { "$SYSCRIBE" -m "$1" residual --config "$2" --json 2>/dev/null; }

    _scn "a required parameter whose window opens after the stage is not W017"
    out=$(run "$B/open")
    [ "$(cnt "$out" W017 "reqrun")" -eq 0 ] && [ "$(cnt "$out" W017 "reqload")" -eq 0 ] && pass "no W017 for load..runtime or runtime windows" || fail "W017 for a later window: $out"
    _scn "a parameter of the Configuration's own stage still warns"
    [ "$(cnt "$out" W017 "reqcomp")" -eq 1 ] && pass "W017 for the compile-window parameter" || fail "expected W017 for reqcomp: $out"
    _scn "the same holds under the --config lens"
    out=$("$SYSCRIBE" -m "$B/open" validate --config CONF-BT-A1 2>&1 || true)
    [ "$(cnt "$out" W017 "reqload")" -eq 0 ] && [ "$(cnt "$out" W017 "reqcomp")" -eq 1 ] && pass "validate --config agrees" || fail "validate --config differs: $out"
}
