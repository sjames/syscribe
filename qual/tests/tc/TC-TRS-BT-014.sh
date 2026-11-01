tc_TRS_BT_014() {
    local F="$1"; local B="$F/TC-TRS-BT-014"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    # has <output> <code>   — the code is reported at all
    has() { printf '%s' "$1" | grep -q "$2"; }
    run() { "$SYSCRIBE" -m "$1" validate 2>&1 || true; }

    _scn "rebinding what an ancestor closed"
    out=$(run "$B/rebind")
    [ "$(cnt "$out" E754 "Motor.kv")" -eq 1 ] && pass "E754 for a different value" || fail "expected one E754: $out"
    out=$(run "$B/same_value")
    has "$out" 'E754' && fail "E754 for an identical value: $out" || pass "restating the same value is silent"
    out=$(run "$B/featclosed")
    [ "$(cnt "$out" E754 "Features::Motor")" -eq 1 ] && pass "E754 for a reversed feature selection" || fail "expected one E754: $out"

    _scn "narrowing only shrinks"
    out=$(run "$B/narrow_ok")
    has "$out" 'E755\|E756\|E754' && fail "finding on a valid narrowing: $out" || pass "a narrower range and an in-range binding are clean"
    out=$(run "$B/widen")
    [ "$(cnt "$out" E755 "widens what")" -eq 1 ] && pass "E755 for widening an ancestor's narrowing" || fail "expected one E755: $out"
    out=$(run "$B/beyond_declared")
    [ "$(cnt "$out" E755 "widens the parameter")" -eq 1 ] && pass "E755 for widening the declared range" || fail "expected one E755: $out"
    out=$(run "$B/unnamed")
    [ "$(cnt "$out" E755 "does not name")" -eq 1 ] && pass "E755 for a narrowing that names no parameter" || fail "expected one E755: $out"

    _scn "a binding must stay inside the narrowed range"
    out=$(run "$B/outside")
    [ "$(cnt "$out" E756 "Motor.kv")" -eq 1 ] && has "$out" 'E205' && fail "E205 as well as E756" || true
    [ "$(cnt "$out" E756 "Motor.kv")" -eq 1 ] && pass "E756 for a binding outside the narrowed range" || fail "expected one E756: $out"
    out=$(run "$B/enum_outside")
    [ "$(cnt "$out" E756 "Motor.mode")" -eq 1 ] && pass "E756 for an enum value outside the narrowed list" || fail "expected one E756: $out"
    out=$(run "$B/enum_ok")
    has "$out" 'E756' && fail "E756 for a value inside the narrowed list" || pass "a value inside the narrowed list is clean"
}
