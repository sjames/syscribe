tc_TRS_VAR_007() {
    local F="$1"; local B="$F/TC-TRS-VAR-007"
    local OK="$B/ok" out rc c

    SCENARIO_NAME="an inheriting Configuration validates cleanly"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$OK" validate 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "validate exits 0" || fail "validate exit $rc"
    for c in E105 E017 W016 W017 E203 E215; do
        grep -qF "| $c |" <<<"$out" && fail "unexpected $c" || pass "no $c"
    done

    SCENARIO_NAME="projection uses the effective selection"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$OK" list Requirement --config CONF-V7-CHILD-001 2>/dev/null || true)
    grep -qF "REQ-V7-WDT-001" <<<"$out" && grep -qF "REQ-V7-LOG-001" <<<"$out" \
        && pass "child adding Log shows the inherited Wdt and its own Log requirement" \
        || fail "child projection missing an inherited or own selection"
    local base bare
    base=$("$SYSCRIBE" -m "$OK" list Requirement --config CONF-V7-BASE-001 2>/dev/null || true)
    bare=$("$SYSCRIBE" -m "$OK" list Requirement --config CONF-V7-BARE-001 2>/dev/null || true)
    [ -n "$base" ] && [ "$base" = "$bare" ] && grep -qF "REQ-V7-WDT-001" <<<"$bare" \
        && pass "bare child projects exactly like its base" || fail "bare child projection differs from base"
    out=$("$SYSCRIBE" -m "$OK" list Requirement --config CONF-V7-NOWDT-001 2>/dev/null || true)
    if grep -qE "REQ-V7-WDT-001|REQ-V7-LOG-001" <<<"$out"; then
        fail "child deselecting Wdt (by id) still projects a gated requirement"
    else
        pass "child deselecting Wdt by feature id projects neither gated requirement"
    fi
    out=$("$SYSCRIBE" -m "$OK" list Requirement --config CONF-V7-GRAND-001 2>/dev/null || true)
    grep -qF "REQ-V7-LOG-001" <<<"$out" && grep -qF "REQ-V7-WDT-001" <<<"$out" \
        && pass "grandchild inherits through two levels" || fail "grandchild lost an inherited selection"

    SCENARIO_NAME="matrix, configure, show and all-configs use the effective selection"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$OK" matrix --json 2>/dev/null || true)
    local cell
    cell=$(jq -r '.rows[] | select(.id=="REQ-V7-WDT-001") | .cells["CONF-V7-BARE-001"]' <<<"$out" 2>/dev/null || true)
    [ "$cell" = "gap" ] && pass "matrix: Wdt requirement applicable in the bare child's column" || fail "matrix cell for bare child is '$cell' (expected gap)"
    cell=$(jq -r '.rows[] | select(.id=="REQ-V7-WDT-001") | .cells["CONF-V7-NOWDT-001"]' <<<"$out" 2>/dev/null || true)
    [ "$cell" = "na" ] && pass "matrix: Wdt requirement N/A where the child deselects Wdt" || fail "matrix cell for no-Wdt child is '$cell' (expected na)"
    out=$("$SYSCRIBE" -m "$OK" configure CONF-V7-BARE-001 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && grep -qF -- "- free: none" <<<"$out" \
        && pass "configure completes the bare child from its inherited selection" || fail "configure on the bare child (exit $rc) left features free"
    out=$("$SYSCRIBE" -m "$OK" show CONF-V7-CHILD-001 2>/dev/null || true)
    grep -qF "| Features::Wdt | true (inherited) |" <<<"$out" && grep -qF "| Features::Log | true |" <<<"$out" \
        && pass "show marks inherited entries and keeps own entries unmarked" || fail "show does not mark inherited entries"
    "$SYSCRIBE" -m "$OK" validate --all-configs >/dev/null 2>&1 && pass "validate --all-configs exits 0" || fail "validate --all-configs failed"
    "$SYSCRIBE" -m "$OK" validate --config CONF-V7-GRAND-001 >/dev/null 2>&1 && pass "validate --config <grandchild> exits 0" || fail "validate --config <grandchild> failed"

    SCENARIO_NAME="a child's own binding overrides the inherited one"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B/override" validate 2>&1 || true)
    grep -E "^\| E205 \|" <<<"$out" | grep -qF "CONF-V7-OVR-001.md" && pass "E205 raised on the overriding child" || fail "no E205 on the overriding child"
    grep -E "^\| E205 \|" <<<"$out" | grep -qF "CONF-V7-BASE-001.md" && fail "E205 wrongly raised on the base" || pass "base binding unaffected"

    SCENARIO_NAME="a consolidated peer Configuration inherits the binding that closes its parameter"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B/hple/top" validate 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "consolidating tier validates (exit 0)" || fail "consolidating tier exit $rc"
    grep -qF "| E518 |" <<<"$out" && fail "E518: inheriting peer Configuration judged invalid" || pass "no E518"
    grep -qF "| W513 |" <<<"$out" && fail "W513: inherited binding not credited" || pass "no W513"

    SCENARIO_NAME="structural problems are reported with dedicated codes"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B/bad" validate 2>&1 || true)
    grep -E "^\| E234 \|" <<<"$out" | grep -qF "CONF-V7-DANG-001.md" && pass "E234 dangling base" || fail "E234 missing"
    grep -E "^\| E235 \|" <<<"$out" | grep -qF "CONF-V7-WRONG-001.md" && pass "E235 non-Configuration base" || fail "E235 missing"
    [ "$(grep -cE '^\| E236 \|' <<<"$out")" -eq 2 ] && pass "E236 on both cycle members" || fail "E236 not raised on both cycle members"
    grep -E "^\| E237 \|" <<<"$out" | grep -qF "CONF-V7-MULTI-001.md" && pass "E237 multiple bases" || fail "E237 missing"
    grep -E "^\| E215 \|" <<<"$out" | grep -qF "CONF-V7-KID-001.md" && pass "E215 draft base" || fail "E215 missing"
    grep -qE "^\| (E105|E017) \|" <<<"$out" && fail "E105/E017 raised for a Configuration derivedFrom" || pass "no E105/E017"
}
