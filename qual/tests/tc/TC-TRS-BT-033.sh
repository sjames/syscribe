tc_TRS_BT_033() {
    local F="$1"; local B="$F/TC-TRS-BT-031/stage"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    has() { printf '%s' "$1" | grep -q "$2"; }
    val() { "$SYSCRIBE" -m "$1" validate --config "$2" 2>&1 || true; }

    _scn "residual and validate --config list the same retained elements"
    a=$("$SYSCRIBE" -m "$B" residual --config CONF-BT-A1 --json 2>/dev/null | jq -r '[.retained[].id]|sort|join(" ")')
    b=$(val "$B" CONF-BT-A1 | grep I750 | grep -o "'[A-Z]*-[A-Z]*-[0-9]*'" | tr -d "'" | sort | tr '\n' ' ' | sed 's/ $//')
    [ -n "$a" ] && [ "$a" = "$b" ] && pass "both list: $a" || fail "lists differ: residual [$a] vs validate [$b]"
    out=$("$SYSCRIBE" -m "$B" residual --config CONF-BT-A1 2>&1)
    has "$out" 'Variant-retained elements' && pass "residual text has the section" || fail "no section in residual text"
    "$SYSCRIBE" -m "$B" residual --config CONF-BT-A1 --json 2>/dev/null | jq -e '.reliances|length==1' >/dev/null && pass "reliances are in the JSON too" || fail "reliances missing"

    _scn "an unstaged Configuration is projected exactly as before"
    local U="$F/TC-TRS-BT-033/unstaged"
    out=$("$SYSCRIBE" -m "$U" validate --config CONF-BT-A1 2>&1 || true); has "$out" 'I750\|W752' && fail "retention findings on an unstaged Configuration: $out" || pass "no retention findings"
    [ "$("$SYSCRIBE" -m "$U" list Requirement --config CONF-BT-A1 2>/dev/null | grep -o 'REQ-[A-Z]*-[0-9]*' | sort -u | tr '\n' ' ')" = "REQ-BASE-001 REQ-NOSENSOR-001 " ] \
        && pass "Sensor unselected: the two-valued projection drops REQ-SENSOR-001" || fail "unstaged projection changed"
    "$SYSCRIBE" -m "$U" residual --config CONF-BT-A1 --json 2>/dev/null | jq -e '.retained|length==0' >/dev/null && pass "residual retains nothing" || fail "residual retained elements"
}
