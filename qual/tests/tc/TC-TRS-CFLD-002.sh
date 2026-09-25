tc_TRS_CFLD_002() {
    local F="$1"; local M="$F/TC-TRS-CFLD-002/model"
    local out

    SCENARIO_NAME="exact match: custom.supplier=Bosch"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" ls --where custom.supplier=Bosch 2>/dev/null || true)
    printf '%s' "$out" | grep -q "Engine" && pass "matches Engine" || fail "Engine not matched"
    printf '%s' "$out" | grep -q "Gearbox" && fail "Gearbox wrongly matched" || pass "Gearbox excluded"

    SCENARIO_NAME="substring/regex: custom.costCenter=~PWT"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" ls --where custom.costCenter=~PWT 2>/dev/null || true)
    printf '%s' "$out" | grep -q "Engine" && printf '%s' "$out" | grep -q "Gearbox" \
        && pass "matches both PWT cost centers" || fail "substring match wrong"

    SCENARIO_NAME="list membership: custom.partNumbers~=A-1001"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" ls --where custom.partNumbers~=A-1001 2>/dev/null || true)
    printf '%s' "$out" | grep -q "Engine" && pass "matches list member" || fail "list membership failed"
    printf '%s' "$out" | grep -q "Gearbox" && fail "Gearbox wrongly matched" || pass "non-member excluded"

    SCENARIO_NAME="presence: custom.supplier"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" ls --where custom.supplier 2>/dev/null || true)
    printf '%s' "$out" | grep -q "Engine" && printf '%s' "$out" | grep -q "Gearbox" \
        && pass "presence matches both" || fail "presence match wrong"
    printf '%s' "$out" | grep -q "Wheel" && fail "Wheel (no custom fields) wrongly matched" \
        || pass "element without the field excluded"

    SCENARIO_NAME="unparseable predicate exits non-zero"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$M" ls --where 'bogus!!pred' >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "bad predicate errors" || fail "bad predicate did not error"

    SCENARIO_NAME="unsupported operators are usage errors (issue #129)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local c o e
    while IFS= read -r c; do
        o=$(eval "\"\$SYSCRIBE\" -m \"\$M\" $c" 2>/dev/null) && rc=0 || rc=$?
        e=$(eval "\"\$SYSCRIBE\" -m \"\$M\" $c" 2>&1 >/dev/null || true)
        if [ "$rc" -eq 1 ] && [ -z "$o" ] && grep -qF "unsupported --where operator" <<<"$e" \
            && grep -qF "=~" <<<"$e" && grep -qF "~=" <<<"$e"; then
            pass "$c → exit 1, lists supported operators"
        else
            fail "$c → exit $rc, stdout ${#o} bytes, stderr: $(head -c 160 <<<"$e")"
        fi
    done <<'CMDS'
list PartDef --where custom.supplier!=Bosch
list PartDef --where custom.supplier==Bosch
ls --where 'custom.supplier~Bosch'
ls --where 'custom.mass>5'
ls --where 'custom.mass<5'
find . --where 'custom.mass>=5'
find . --where 'custom.mass<=5'
CMDS
    out=$("$SYSCRIBE" -m "$M" list PartDef --where custom.supplier=Bosch 2>/dev/null || true)
    grep -q "Engine" <<<"$out" && ! grep -q "Gearbox" <<<"$out" \
        && pass "list PartDef --where custom.supplier=Bosch still filters" || fail "list --where exact match broken"
    out=$("$SYSCRIBE" -m "$M" find . --where custom.maturity=prototype 2>/dev/null || true)
    grep -q "Engine" <<<"$out" && ! grep -q "Gearbox" <<<"$out" \
        && pass "find . --where custom.maturity=prototype still filters" || fail "find --where exact match broken"
}
