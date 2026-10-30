tc_TRS_BT_016() {
    local F="$1"; local B="$F/TC-TRS-BT-016"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    has() { printf '%s' "$1" | grep -q "$2"; }
    run() { "$SYSCRIBE" -m "$1" validate 2>&1 || true; }
    res() { "$SYSCRIBE" -m "$1" residual --config "$2" --json 2>/dev/null; }

    _scn "a compile Configuration leaves the later-stage items open"
    out=$(res "$B/chain" CONF-BT-A1)
    printf '%s' "$out" | jq -e '[.groups[]|select(.stage=="load")|.items[].key]|sort==["Features::Motor.spin","Features::Sensor"]' >/dev/null \
        && pass "load: Sensor and Motor.spin" || fail "load group wrong: $out"
    printf '%s' "$out" | jq -e '[.groups[]|select(.stage=="runtime")|.items[].key]==["Features::Motor.gain"]' >/dev/null \
        && pass "runtime: Motor.gain" || fail "runtime group wrong"
    printf '%s' "$out" | jq -e '[.groups[]|select(.stage=="compile")|.items[].key]|sort==["Features::Motor.cc","Features::Radio"]' >/dev/null \
        && pass "compile: still bindable now" || fail "compile group wrong"
    printf '%s' "$out" | jq -e '[.groups[].items[].key]|(index("Features::Motor.kv")==null) and (index("Features::Motor.lim")==null)' >/dev/null \
        && pass "bound and fixed items are not listed" || fail "a bound or fixed item is listed"

    _scn "an item whose window closed is listed apart"
    out=$(res "$B/chain" CONF-BT-B1)
    printf '%s' "$out" | jq -e '[.groups[]|select(.closed)|.items[].key]|sort==["Features::Motor.cc","Features::Radio"]' >/dev/null \
        && pass "closed: compile-only Radio and Motor.cc at a load Configuration" || fail "closed group wrong: $out"

    _scn "output is stable and sorted"
    [ "$(res "$B/chain" CONF-BT-A1)" = "$(res "$B/chain" CONF-BT-A1)" ] && pass "identical across runs" || fail "output differs between runs"
    printf '%s' "$(res "$B/chain" CONF-BT-A1)" | jq -e '[.groups[]|select(.closed|not)|.stage]==["compile","load","runtime"]' >/dev/null \
        && pass "groups ordered earliest to latest" || fail "groups not ordered"
    rc=0; "$SYSCRIBE" -m "$B/chain" residual --config CONF-NOPE >/dev/null 2>&1 || rc=$?
    [ "$rc" -ne 0 ] && pass "an unknown Configuration fails" || fail "unknown Configuration accepted"
    out=$("$SYSCRIBE" -m "$B/chain" residual --config CONF-BT-A1 2>&1)
    printf '%s' "$out" | grep -q 'Next bindable at load' && pass "text output groups by stage" || fail "text output wrong: $out"
}
