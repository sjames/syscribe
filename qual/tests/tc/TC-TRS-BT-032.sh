tc_TRS_BT_032() {
    local F="$1"; local B="$F/TC-TRS-BT-031/stage"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    has() { printf '%s' "$1" | grep -q "$2"; }
    val() { "$SYSCRIBE" -m "$1" validate --config "$2" 2>&1 || true; }

    _scn "a test that applies only where the retained requirement is absent is reported"
    out=$(val "$B" CONF-BT-A1)
    [ "$(cnt "$out" W752 "TC-NOSENSOR-001")" -eq 1 ] && pass "W752 names the dependent" || fail "expected one W752 for TC-NOSENSOR-001: $out"
    printf '%s' "$out" | grep W752 | grep -q 'REQ-SENSOR-001' && pass "and the retained element it relies on" || fail "W752 does not name the target"

    _scn "a test that does not rely on absence is silent"
    [ "$(cnt "$out" W752 "TC-OK-001")" -eq 0 ] && pass "TC-OK-001 is not reported" || fail "TC-OK-001 reported"
    [ "$(printf '%s' "$out" | grep -c W752)" -eq 1 ] && pass "exactly one reliance" || fail "unexpected W752 count: $out"

    _scn "nothing to rely on once the stage has decided"
    out=$(val "$B" CONF-BT-R0); has "$out" W752 && fail "W752 at a decided Configuration" || pass "no W752"
}
