tc_TRS_QNAME_005() {
    local F="$1"; local B="$F/TC-TRS-QNAME-005"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out w049 n show

    out=$("$SYSCRIBE" -m "$B/model" validate 2>/dev/null) || true
    w049=$(grep -F "| W049 |" <<<"$out" || true)

    _scn "a differing qualifiedName is reported and ignored"
    n=$(grep -c . <<<"$w049" || true)
    [ -n "$w049" ] && [ "${n:-0}" -eq 1 ] && pass "exactly one W049" || fail "W049 count = ${n:-0} (expected 1)"
    grep -F "Pump.md" <<<"$w049" | grep -F "Other::Pump" | grep -qF "Arch::Pump" \
        && pass "W049 names the file, the override and the path-derived name" \
        || fail "W049 does not name Pump.md, Other::Pump and Arch::Pump"
    show=$("$SYSCRIBE" -m "$B/model" show Arch::Pump 2>&1) || true
    grep -qF "# Arch::Pump" <<<"$show" && pass "the element keeps the path-derived name" \
        || fail "show Arch::Pump did not find the element"
    show=$("$SYSCRIBE" -m "$B/model" show Other::Pump 2>&1) || true
    grep -qF "# Other::Pump" <<<"$show" && fail "the override renamed the element" \
        || pass "Other::Pump does not exist"

    _scn "a qualifiedName equal to the path-derived name is harmless"
    grep -qF "Valve.md" <<<"$w049" && fail "W049 on the redundant qualifiedName" \
        || pass "no W049 on Valve.md"
}
