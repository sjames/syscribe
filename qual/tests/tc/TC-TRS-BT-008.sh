tc_TRS_BT_008() {
    local F="$1"; local B="$F/TC-TRS-BT-008"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }

    _scn "a parameter derived from a sibling that binds later"
    out=$("$SYSCRIBE" -m "$B/derived" validate 2>&1 || true)
    [ "$(cnt "$out" E229 "Motor.a")" -eq 1 ] && pass "E229 for the dependent parameter" || fail "expected one E229: $out"
    out=$("$SYSCRIBE" -m "$B/derived_ok" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E229' && fail "E229 for the reverse ordering" || pass "the reverse ordering is clean"
    out=$("$SYSCRIBE" -m "$B/derived_overlap" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E229' && fail "E229 for overlapping windows" || pass "overlapping windows are feasible and silent"
    out=$("$SYSCRIBE" -m "$B/derived_unspecified" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E229' && fail "E229 on an edge with an unspecified end" || pass "an unspecified end is not checked"

    _scn "a parameter bound to a later-binding system parameter"
    out=$("$SYSCRIBE" -m "$B/bindto" validate 2>&1 || true)
    [ "$(cnt "$out" E229 "Comp.kv")" -eq 1 ] && pass "E229 for the bindTo edge" || fail "expected one E229: $out"
    out=$("$SYSCRIBE" -m "$B/bindto_ok" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E229' && fail "E229 for a feasible bindTo" || pass "a feasible bindTo is clean"

    _scn "a feature that requires a feature whose window opens later"
    out=$("$SYSCRIBE" -m "$B/feature" validate 2>&1 || true)
    [ "$(cnt "$out" E229 "requires")" -eq 1 ] && pass "E229 for the requires edge" || fail "expected one E229: $out"
    out=$("$SYSCRIBE" -m "$B/feature_overlap" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E229' && fail "E229 for overlapping feature windows" || pass "overlapping feature windows are silent"
    out=$("$SYSCRIBE" -m "$B/feature_unspecified" validate 2>&1 || true)
    printf '%s' "$out" | grep -q 'E229' && fail "E229 with an unspecified end" || pass "an unspecified feature end is not checked"
}
