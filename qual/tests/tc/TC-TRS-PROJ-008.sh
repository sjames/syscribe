tc_TRS_PROJ_008() {
    local F="$1"; local B="$F/TC-TRS-PROJ-008/lens"
    local NO="CONF-P8-NOWDT-001" YES="CONF-P8-WDT-001"
    local c o rc e

    SCENARIO_NAME="inactive satisfiers and verifiers disappear under the lens"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    while IFS= read -r c; do
        o=$(eval "\"\$SYSCRIBE\" -m \"\$B\" $c --config $NO" 2>/dev/null) && rc=0 || rc=$?
        if [ "$rc" -eq 0 ] && grep -qF "TC-P8-CORE-001" <<<"$o"; then
            pass "$c --config $NO → exit 0, active verifier listed"
        else
            fail "$c --config $NO → exit $rc or active verifier missing"
        fi
        if grep -qE "WatchdogDriver|TC-P8-WDT-001" <<<"$o"; then
            fail "$c --config $NO still lists a Wdt-gated element"
        else
            pass "$c --config $NO omits the Wdt-gated part and TestCase"
        fi
        o=$(eval "\"\$SYSCRIBE\" -m \"\$B\" $c --config $YES" 2>/dev/null || true)
        grep -qF "TC-P8-WDT-001" <<<"$o" && pass "$c --config $YES lists the Wdt-gated TestCase" || fail "$c --config $YES lost the Wdt-gated TestCase"
        o=$(eval "\"\$SYSCRIBE\" -m \"\$B\" $c" 2>/dev/null || true)
        grep -qF "TC-P8-WDT-001" <<<"$o" && pass "$c without --config shows the whole model" || fail "$c without --config lost the Wdt-gated TestCase"
    done <<'CMDS'
trace REQ-P8-CORE-001
who-verifies REQ-P8-CORE-001
refs REQ-P8-CORE-001
links REQ-P8-CORE-001
CMDS
    o=$("$SYSCRIBE" -m "$B" trace REQ-P8-CORE-001 --config "$NO" 2>/dev/null || true)
    grep -qF "WatchdogDriver" <<<"$o" && fail "trace still lists the inactive satisfier" || pass "trace omits the inactive satisfier"

    SCENARIO_NAME="why reads the projected view"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    o=$("$SYSCRIBE" -m "$B" why Arch::CoreMonitor --config "$NO" 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && grep -qF "REQ-P8-CORE-001" <<<"$o" && pass "why --config $NO → exit 0, names the satisfied requirement" || fail "why --config $NO → exit $rc"
    grep -qF "TC-P8-WDT-001" <<<"$o" && fail "why --config $NO lists the inactive TestCase" || pass "why --config $NO omits the inactive TestCase"
    o=$("$SYSCRIBE" -m "$B" why Arch::CoreMonitor 2>/dev/null || true)
    grep -qF "TC-P8-WDT-001" <<<"$o" && pass "why without --config lists every verifier" || fail "why without --config lost the Wdt-gated TestCase"

    SCENARIO_NAME="an inactive start element is a usage error"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    while IFS= read -r c; do
        o=$(eval "\"\$SYSCRIBE\" -m \"\$B\" $c --config $NO" 2>/dev/null) && rc=0 || rc=$?
        e=$(eval "\"\$SYSCRIBE\" -m \"\$B\" $c --config $NO" 2>&1 >/dev/null || true)
        if [ "$rc" -eq 1 ] && [ -z "$o" ] && grep -qF "is not active in configuration '$NO'" <<<"$e"; then
            pass "$c --config $NO → exit 1, no stdout, names the inactive element"
        else
            fail "$c --config $NO → exit $rc, stdout ${#o} bytes, stderr: $(head -c 160 <<<"$e")"
        fi
        eval "\"\$SYSCRIBE\" -m \"\$B\" $c --config $YES" >/dev/null 2>&1 && pass "$c --config $YES → exit 0" || fail "$c --config $YES failed"
    done <<'CMDS'
trace REQ-P8-WDT-001
who-verifies REQ-P8-WDT-001
refs REQ-P8-WDT-001
links Reqs::REQ-P8-WDT-001
why Arch::WatchdogDriver
CMDS

    SCENARIO_NAME="an unresolvable --config is a usage error"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B" trace REQ-P8-CORE-001 --config CONF-P8-NOPE-001 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "unresolvable --config → exit 1" || fail "unresolvable --config → exit $rc"
}
