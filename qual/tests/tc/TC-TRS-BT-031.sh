tc_TRS_BT_031() {
    local F="$1"; local B="$F/TC-TRS-BT-031/stage"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    cnt() { printf '%s' "$1" | grep "$2" | grep -c -- "$3" || true; }
    has() { printf '%s' "$1" | grep -q "$2"; }
    val() { "$SYSCRIBE" -m "$1" validate --config "$2" 2>&1 || true; }

    ids() { "$SYSCRIBE" -m "$B" list Requirement --config "$1" 2>/dev/null | grep -o 'REQ-[A-Z]*-[0-9]*' | sort -u | tr '\n' ' '; }

    _scn "a requirement conditioned on a still-open feature is retained at a compile Configuration"
    [ "$(ids CONF-BT-A1)" = "REQ-BASE-001 REQ-NOSENSOR-001 REQ-SENSOR-001 " ] && pass "both Sensor-conditioned requirements are kept" || fail "projection wrong: $(ids CONF-BT-A1)"
    out=$(val "$B" CONF-BT-A1)
    [ "$(cnt "$out" I750 "REQ-SENSOR-001")" -eq 1 ] && [ "$(cnt "$out" I750 "REQ-NOSENSOR-001")" -eq 1 ] && pass "each is reported variant-retained" || fail "I750 missing: $out"
    [ "$(cnt "$out" I750 "REQ-BASE-001")" -eq 0 ] && pass "an unconditioned requirement is not" || fail "REQ-BASE-001 reported retained"

    _scn "it is removed once a runtime Configuration decides the feature off"
    [ "$(ids CONF-BT-R0)" = "REQ-BASE-001 REQ-NOSENSOR-001 " ] && pass "Sensor off: REQ-SENSOR-001 removed" || fail "R0 projection wrong: $(ids CONF-BT-R0)"
    [ "$(ids CONF-BT-R1)" = "REQ-BASE-001 REQ-SENSOR-001 " ] && pass "Sensor on: REQ-NOSENSOR-001 removed" || fail "R1 projection wrong: $(ids CONF-BT-R1)"
    out=$(val "$B" CONF-BT-R0); has "$out" I750 && fail "I750 at a decided Configuration" || pass "nothing retained once decided"
}
