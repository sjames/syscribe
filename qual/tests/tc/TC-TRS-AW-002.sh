tc_TRS_AW_002() {
    local F="$1"; local B="$F/TC-TRS-AW-002/model"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. own gate is shown
    _scn "an element's own appliesWhen is displayed"
    out=$("$SYSCRIBE" -m "$B" applies-when REQ-AW2-OWN-001 2>&1)
    { printf '%s' "$out" | grep -qE 'own:[[:space:]]+FEAT-ABS' \
        && printf '%s' "$out" | grep -qE 'effective:[[:space:]]+FEAT-ABS'; } \
        && pass "own gate FEAT-ABS shown" || fail "own gate not shown"

    # 2. inherited (package) gate is shown as effective
    _scn "a gate inherited from an ancestor package is shown as effective"
    out=$("$SYSCRIBE" -m "$B" applies-when REQ-AW2-INH-001 2>&1)
    { printf '%s' "$out" | grep -qE 'own:[[:space:]]+\(none\)' \
        && printf '%s' "$out" | grep -qiF 'inherited from package' \
        && printf '%s' "$out" | grep -qF 'Optional'; } \
        && pass "inherited package gate shown as effective" || fail "inherited gate not shown"

    # 3. an ungated element reports it always applies
    _scn "an ungated element reports always applies"
    out=$("$SYSCRIBE" -m "$B" applies-when REQ-AW2-PLAIN-001 2>&1)
    printf '%s' "$out" | grep -qiF 'always applies' \
        && pass "ungated element reports always applies" || fail "ungated element not reported"

    # 4. read mode is read-only (file unchanged)
    _scn "read mode does not modify the file"
    local before after
    before=$(md5sum "$B/Reqs/Own.md" | cut -d' ' -f1)
    "$SYSCRIBE" -m "$B" applies-when REQ-AW2-OWN-001 >/dev/null 2>&1 || true
    after=$(md5sum "$B/Reqs/Own.md" | cut -d' ' -f1)
    [ "$before" = "$after" ] && pass "file unchanged by read mode" || fail "read mode modified the file"

    # 5. --json emits the structured form
    _scn "--json emits a structured object"
    out=$("$SYSCRIBE" -m "$B" applies-when REQ-AW2-INH-001 --json 2>&1)
    { printf '%s' "$out" | grep -qF '"effective"' \
        && printf '%s' "$out" | grep -qF '"inheritedFrom"'; } \
        && pass "json carries effective + inheritedFrom" || fail "json form missing fields"

    # 6. an unresolved element exits non-zero
    _scn "an unresolved element exits non-zero"
    "$SYSCRIBE" -m "$B" applies-when REQ-NOPE-001 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "unknown element exits non-zero" || fail "unknown element did not fail"
}
