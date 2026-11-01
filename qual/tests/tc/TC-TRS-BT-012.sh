tc_TRS_BT_012() {
    local F="$1"; local B="$F/TC-TRS-BT-012"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    # has <output> <code>   — the code is reported at all
    has() { printf '%s' "$1" | grep -q "$2"; }
    run() { "$SYSCRIBE" -m "$1" validate 2>&1 || true; }

    _scn "a compile-stage Configuration binding and selecting later-window items"
    out=$(run "$B/early")
    [ "$(cnt "$out" E752 "parameterBindings binds")" -eq 1 ] && pass "E752 for the parameter binding" || fail "expected one parameter E752: $out"
    [ "$(cnt "$out" E752 "selects feature")" -eq 1 ] && pass "E752 for the feature selection" || fail "expected one feature E752: $out"

    _scn "items whose window contains the stage are clean"
    out=$(run "$B/inwindow")
    has "$out" 'E752' && fail "E752 inside the window: $out" || pass "no E752"
    out=$(run "$B/runtime_ok")
    has "$out" 'E752' && fail "E752 for a runtime parameter at runtime" || pass "a runtime parameter may be bound at the runtime stage"
    has "$out" 'W027' && fail "W027 on a staged Configuration" || pass "W027 does not apply to a staged Configuration"

    _scn "a runtime parameter cannot be bound before runtime"
    out=$(run "$B/runtime_early")
    [ "$(cnt "$out" E752 "Motor.gain")" -eq 1 ] && pass "E752 at stage load" || fail "expected E752: $out"

    _scn "an inherited binding is judged where it was made, not at the derived stage"
    out=$(run "$B/inherited")
    has "$out" 'E752' && fail "E752 for an inherited binding: $out" || pass "a runtime Configuration may inherit a compile-stage binding"
}

