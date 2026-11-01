tc_TRS_BT_018() {
    local F="$1"; local B="$F/TC-TRS-BT-018"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # cnt <output> <code> <substring>  — findings with that code mentioning the substring
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    has() { printf '%s' "$1" | grep -q "$2"; }
    run() { "$SYSCRIBE" -m "$1" validate 2>&1 || true; }
    res() { "$SYSCRIBE" -m "$1" residual --config "$2" --json 2>/dev/null; }

    local plain staged resid
    _scn "items whose window excludes the stage are not listed as choices"
    plain=$("$SYSCRIBE" -m "$B/chain" configure CONF-BT-A1 --json 2>/dev/null)
    staged=$("$SYSCRIBE" -m "$B/chain" configure CONF-BT-A1 --stage compile --json 2>/dev/null)
    printf '%s' "$plain" | jq -e '(.free|index("Features::Sensor"))!=null' >/dev/null && pass "without --stage Sensor is a free choice" || fail "Sensor missing without --stage: $plain"
    printf '%s' "$staged" | jq -e '(.free|index("Features::Sensor"))==null and .stage=="compile"' >/dev/null && pass "--stage compile drops the load-window Sensor" || fail "Sensor still listed: $staged"
    printf '%s' "$staged" | jq -e '(.free|index("Features::Radio"))!=null' >/dev/null && pass "the compile-window Radio is a compile-stage choice" || fail "Radio missing: $staged"

    _scn "output agrees with residual"
    resid=$(res "$B/chain" CONF-BT-A1 | jq -c '[.groups[]|select(.closed|not)|.items[]|select(.kind=="feature")|.key]')
    printf '%s' "$staged" | jq -e --argjson r "$resid" '.free|all(. as $f|$r|index($f)!=null)' >/dev/null \
        && pass "every free choice at the stage is an open residual feature" || fail "free choices not in residual: $staged vs $resid"
    rc=0; "$SYSCRIBE" -m "$B/chain" configure CONF-BT-A1 --stage bogus >/dev/null 2>&1 || rc=$?
    [ "$rc" -eq 2 ] && pass "an unknown stage is a usage error" || fail "unknown stage accepted (exit $rc)"
}
