tc_TRS_DISC_007() {
    local F="$1"; local B="$F/TC-TRS-DISC-007/pl"
    local out ec

    SCENARIO_NAME="list --status narrows by lifecycle status"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B" list Requirement --status draft 2>/dev/null)
    printf '%s' "$out" | grep -qF "REQ-D7-DRAFT-001" && pass "draft requirement listed" || fail "draft requirement missing"
    printf '%s' "$out" | grep -qF "REQ-D7-CORE-001" && fail "approved requirement wrongly listed" || pass "approved requirement excluded"

    SCENARIO_NAME="list --sil matches a numeric SIL level"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B" list Requirement --sil 4 2>/dev/null)
    printf '%s' "$out" | grep -qF "REQ-D7-DRAFT-001" && pass "silLevel 4 requirement listed" || fail "silLevel 4 requirement missing"
    printf '%s' "$out" | grep -qF "REQ-D7-CORE-001" && fail "non-SIL-4 requirement wrongly listed" || pass "non-SIL-4 requirement excluded"

    SCENARIO_NAME="list --sil matches an ASIL letter"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B" list Requirement --sil D 2>/dev/null)
    printf '%s' "$out" | grep -qF "REQ-D7-WDT-001" && pass "asilLevel D requirement listed" || fail "asilLevel D requirement missing"
    printf '%s' "$out" | grep -qF "REQ-D7-DRAFT-001" && fail "non-ASIL-D requirement wrongly listed" || pass "non-ASIL-D requirement excluded"

    SCENARIO_NAME="list --json emits a JSON array reflecting a filter"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B" list Requirement --status draft --json 2>/dev/null) && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "list --json exits 0" || fail "list --json exit ${ec}"
    if command -v jq >/dev/null 2>&1; then
        printf '%s' "$out" | jq -e 'type == "array" and length == 1 and .[0].status == "draft" and .[0].id == "REQ-D7-DRAFT-001"' >/dev/null 2>&1 \
            && pass "JSON array reflects the status filter" || fail "JSON did not reflect the filter"
    else
        printf '%s' "$out" | grep -qF '"status": "draft"' && printf '%s' "$out" | grep -qF '[' \
            && pass "JSON array contains the filtered status" || fail "JSON missing array/status"
    fi

    SCENARIO_NAME="matrix --status restricts rows"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B" matrix --status approved 2>/dev/null)
    printf '%s' "$out" | grep -qF "REQ-D7-CORE-001" && pass "approved row present" || fail "approved row missing"
    printf '%s' "$out" | grep -qF "REQ-D7-DRAFT-001" && fail "draft row wrongly present" || pass "draft row excluded"

    SCENARIO_NAME="matrix --gaps-only keeps only rows with a gap"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B" matrix --gaps-only 2>/dev/null)
    printf '%s' "$out" | grep -qF "REQ-D7-GAP-001" && pass "gap row retained" || fail "gap row dropped"
    printf '%s' "$out" | grep -qF "REQ-D7-CORE-001" && fail "fully-covered row wrongly retained" || pass "fully-covered row dropped"

    SCENARIO_NAME="matrix prints a coverage footer"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B" matrix 2>/dev/null)
    printf '%s' "$out" | grep -qiE "Overall: .*%" && pass "overall coverage percentage printed" || fail "no overall coverage percentage"

    SCENARIO_NAME="matrix --json carries a coverage object"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$B" matrix --json 2>/dev/null)
    if command -v jq >/dev/null 2>&1; then
        printf '%s' "$out" | jq -e '.coverage.overall.pct != null and (.coverage.perConfig | length) >= 1' >/dev/null 2>&1 \
            && pass "coverage object has perConfig and overall.pct" || fail "coverage object malformed"
    else
        printf '%s' "$out" | grep -qF '"coverage"' && printf '%s' "$out" | grep -qF '"perConfig"' \
            && printf '%s' "$out" | grep -qF '"overall"' && pass "coverage object present" || fail "coverage object missing"
    fi
}
