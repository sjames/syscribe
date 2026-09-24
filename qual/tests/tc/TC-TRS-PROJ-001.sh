tc_TRS_PROJ_001() {
    local F="$1"; local B="$F/TC-TRS-PROJ-001/lens"
    local NOFM="$F/TC-TRS-FM-001/no-fm"

    SCENARIO_NAME="lens filters to active elements (stored config)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local w; w=$("$SYSCRIBE" -m "$B" list Requirement --config CONF-PROJ1-WDT-001 2>/dev/null || true)
    printf '%s' "$w" | grep -qF "REQ-PROJ1-CORE-001" && pass "core req present in Wdt variant" || fail "core req missing"
    printf '%s' "$w" | grep -qF "REQ-PROJ1-WDT-001" && pass "Wdt req present in Wdt variant" || fail "Wdt req missing in Wdt variant"
    local n; n=$("$SYSCRIBE" -m "$B" list Requirement --config CONF-PROJ1-NOWDT-001 2>/dev/null || true)
    printf '%s' "$n" | grep -qF "REQ-PROJ1-WDT-001" && fail "Wdt req wrongly present in no-Wdt variant" || pass "Wdt req excluded in no-Wdt variant"

    SCENARIO_NAME="ad-hoc feature selection"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local a; a=$("$SYSCRIBE" -m "$B" list Requirement --config "Features::Wdt" 2>/dev/null || true)
    printf '%s' "$a" | grep -qF "REQ-PROJ1-WDT-001" && pass "ad-hoc selection activates Wdt req" || fail "ad-hoc selection did not work"

    SCENARIO_NAME="--config is a usage error with no feature model"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$NOFM" list Requirement >/dev/null 2>&1 && pass "without --config the command succeeds" || fail "plain list failed on the no-feature-model fixture"
    local c o e rc
    while IFS= read -r c; do
        o=$(eval "\"\$SYSCRIBE\" -m \"\$NOFM\" $c" 2>/dev/null) && rc=0 || rc=$?
        e=$(eval "\"\$SYSCRIBE\" -m \"\$NOFM\" $c" 2>&1 >/dev/null || true)
        if [ "$rc" -ne 0 ] && [ -z "$o" ] && printf '%s' "$e" | grep -q "declares no feature model"; then
            pass "$c → exit $rc, no stdout, names the missing feature model"
        else
            fail "$c → exit $rc, stdout ${#o} bytes, stderr: $(printf '%s' "$e" | head -c 160)"
        fi
    done <<'CMDS'
list Requirement --config CONF-X
validate --config CONF-X
export --config CONF-X
audit --config CONF-X
stats --config CONF-X
digest --config CONF-X
summarize --config CONF-X
search-text fixture --config CONF-X
topics --config CONF-X
clusters --config CONF-X
why-active REQ-FM1-NOFM-001 --config CONF-X
diff --config CONF-X --config CONF-Y
CMDS

    SCENARIO_NAME="--config naming a stored Configuration is accepted with no feature model"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local MG="$F/TC-TRS-MG-011/ok" p1 p2 rc1
    p1=$("$SYSCRIBE" -m "$MG" list Requirement 2>/dev/null) || true
    p2=$("$SYSCRIBE" -m "$MG" list Requirement --config CONF-MG-VARIANT-001 2>/dev/null) && rc1=0 || rc1=$?
    [ "$rc1" -eq 0 ] && [ -n "$p1" ] && [ "$p1" = "$p2" ] && pass "list --config <stored parametric variant> = whole model, exit 0" || fail "list --config CONF-MG-VARIANT-001 rc=$rc1, identical=$([ "$p1" = "$p2" ] && echo yes || echo no)"
    "$SYSCRIBE" -m "$MG" validate --config CONF-MG-VARIANT-001 >/dev/null 2>&1 && pass "validate --config <stored parametric variant> exits 0" || fail "validate --config CONF-MG-VARIANT-001 failed"

    SCENARIO_NAME="unresolved configuration errors"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B" list Requirement --config CONF-DOES-NOT-EXIST >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "unresolved --config exits non-zero" || fail "unresolved --config did not error"
}
