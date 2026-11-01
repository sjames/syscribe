tc_TRS_BT_035() {
    local F="$1"; local B="$F/TC-TRS-BT-035/chain"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    has() { printf '%s' "$1" | grep -q "$2"; }
    val() { "$SYSCRIBE" -m "$1" validate --config "$2" 2>&1 || true; }

    mx() { "$SYSCRIBE" -m "$B" matrix "$@" --json 2>/dev/null; }
    cell() { printf '%s' "$1" | jq -r --arg r "$2" --arg c "$3" '.rows[]|select(.id==$r)|.cells[$c]'; }

    _scn "the grid at compile cannot yet decide what load will"
    c=$(mx --stage compile); l=$(mx --stage load)
    [ "$(cell "$c" REQ-TELEM-001 CONF-BT-L1)" = "open" ] && pass "compile: REQ-TELEM-001 x CONF-BT-L1 is open" || fail "compile cell: $(cell "$c" REQ-TELEM-001 CONF-BT-L1)"
    [ "$(cell "$l" REQ-TELEM-001 CONF-BT-L1)" = "covered" ] && pass "load: it is decided and covered" || fail "load cell: $(cell "$l" REQ-TELEM-001 CONF-BT-L1)"

    _scn "the two stages differ only where a feature is decided between them"
    d=$(jq -nc --argjson a "$c" --argjson b "$l" '[ $a.rows[] as $ra | $b.rows[] | select(.id==$ra.id) | . as $rb | $ra.cells|keys[] | select($ra.cells[.] != $rb.cells[.]) | "\($ra.id) x \(.)" ]|sort')
    [ "$d" = '["REQ-TELEM-001 x CONF-BT-L1"]' ] && pass "only REQ-TELEM-001 x CONF-BT-L1 differs" || fail "unexpected differences: $d"
    [ "$(cell "$c" REQ-BASE-001 CONF-BT-L1)" = "$(cell "$l" REQ-BASE-001 CONF-BT-L1)" ] && pass "an unconditioned requirement is the same at both" || fail "REQ-BASE differs"

    _scn "once the window has closed every Configuration is decided"
    r=$(mx --stage runtime)
    [ "$(cell "$r" REQ-TELEM-001 CONF-BT-C1)" = "na" ] && pass "runtime: the undecided Telemetry is simply off" || fail "runtime cell: $(cell "$r" REQ-TELEM-001 CONF-BT-C1)"

    _scn "no --stage is the grid as authored"
    p=$(mx)
    [ "$(cell "$p" REQ-TELEM-001 CONF-BT-C1)" = "na" ] && [ "$(cell "$p" REQ-TELEM-001 CONF-BT-L1)" = "covered" ] && pass "unchanged" || fail "default grid changed"
    printf '%s' "$p" | jq -e 'has("stage")|not' >/dev/null && printf '%s' "$c" | jq -e '.stage=="compile"' >/dev/null && pass "JSON names the stage only when one is given" || fail "stage key wrong"
    rc=0; "$SYSCRIBE" -m "$B" matrix --stage bogus >/dev/null 2>&1 || rc=$?
    [ "$rc" -eq 2 ] && pass "an unknown stage is a usage error" || fail "unknown stage accepted (exit $rc)"
    out=$("$SYSCRIBE" -m "$B" matrix --stage compile 2>&1); has "$out" '?' && has "$out" 'not yet decided' && pass "text grid shows ? and a legend" || fail "text grid: $out"
}
