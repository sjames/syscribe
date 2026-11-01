tc_TRS_BT_022() {
    local F="$1"; local B="$F/TC-TRS-BT-021"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    pj() { "$SYSCRIBE" -m "$B/chain" configure CONF-BT-B1 --json 2>/dev/null | jq -e "$1" >/dev/null; }

    local show conf js row_show row_conf
    show=$("$SYSCRIBE" -m "$B/chain" show CONF-BT-B1 2>&1)
    conf=$("$SYSCRIBE" -m "$B/chain" configure CONF-BT-B1 2>&1)
    js=$("$SYSCRIBE" -m "$B/chain" configure CONF-BT-B1 --json 2>/dev/null)

    _scn "show lists the binding stage and parameter provenance"
    printf '%s' "$show" | grep -q 'bindingStage:\*\* load' && pass "show prints the stage" || fail "no stage in show: $show"
    row_show=$(printf '%s' "$show" | grep '| Features::Motor.kv |')
    printf '%s' "$row_show" | grep -q 'inherited from CONF-BT-A1' && printf '%s' "$row_show" | grep -q 'compile' \
        && pass "show: kv inherited from CONF-BT-A1 at compile" || fail "show row wrong: $row_show"

    _scn "configure shows the same provenance in text and JSON"
    row_conf=$(printf '%s' "$conf" | grep '| Features::Motor.kv |')
    [ "$row_show" = "$row_conf" ] && pass "show and configure agree on the same value" || fail "rows differ: [$row_show] vs [$row_conf]"
    printf '%s' "$js" | jq -e '.provenance.parameters[]|select(.key=="Features::Motor.kv")|.source.from=="CONF-BT-A1" and .stage=="compile"' >/dev/null \
        && pass "JSON agrees" || fail "JSON differs"
}
