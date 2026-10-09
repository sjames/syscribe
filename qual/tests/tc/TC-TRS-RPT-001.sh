tc_TRS_RPT_001() {
    local F="$1"; local M="$F/TC-TRS-RPT-001/model"

    SCENARIO_NAME="fmea report prints Markdown table with correct column headers"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" fmea report 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "| ID | Failure Mode | Effect | Severity | Occurrence | Detection | RPN | Recommended Action | Status |" \
        && pass "fmea report table has correct headers" \
        || fail "fmea report table missing expected headers"

    SCENARIO_NAME="fmea report rows are sorted by RPN descending"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    first_id=$(printf '%s' "$out" | grep "^| FM-" | head -1 | awk -F'|' '{print $2}' | tr -d ' ')
    [ "$first_id" = "FM-HIGH-001" ] \
        && pass "first row is FM-HIGH-001 (highest RPN 729)" \
        || fail "first row is '$first_id' (expected FM-HIGH-001 with RPN 729)"

    SCENARIO_NAME="fmea report --json emits JSON array with rpn field"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    json=$("$SYSCRIBE" -m "$M" fmea report --json 2>/dev/null || true)
    printf '%s' "$json" | python3 -c "
import json, sys
items = json.load(sys.stdin)
assert isinstance(items, list), 'not a list'
assert all('rpn' in x for x in items), 'some entries missing rpn'
print('ok')
" 2>/dev/null && pass "fmea report --json is a JSON array with rpn fields" || fail "fmea report --json missing rpn"

    SCENARIO_NAME="fmea report --fmea-sheet filters to named sheet"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    filtered=$("$SYSCRIBE" -m "$M" fmea report --fmea-sheet FMEA-REPORT-001 2>/dev/null || true)
    printf '%s' "$filtered" | grep -qF "FM-OTHER-001" \
        && fail "fmea report --fmea-sheet still shows FM-OTHER-001 from other sheet" \
        || pass "fmea report --fmea-sheet excludes entries from other sheets"
    printf '%s' "$filtered" | grep -qF "FM-HIGH-001" \
        && pass "fmea report --fmea-sheet includes FM-HIGH-001 from target sheet" \
        || fail "fmea report --fmea-sheet missing FM-HIGH-001"

    SCENARIO_NAME="fmea report --fmea-sheet <unknown> exits non-zero (GH #218)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$M" fmea report --fmea-sheet NOPE >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -ne 0 ] \
        && pass "unknown --fmea-sheet exits non-zero ($rc)" \
        || fail "unknown --fmea-sheet exited 0"

    local Q="$F/TC-TRS-RPT-001/quality"
    SCENARIO_NAME="FMEA row quality: W931 (no S/O/D), W932 (severity priority), W904 without E115 (GH #218)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    vout=$("$SYSCRIBE" -m "$Q" validate 2>/dev/null || true)
    printf '%s' "$vout" | grep -qF "| W931 |" && pass "W931 for the row with no severity/occurrence/detection" || fail "W931 missing"
    printf '%s' "$vout" | grep -qF "| W932 |" && pass "W932 for severity 10 with RPN 40 and no action" || fail "W932 missing"
    printf '%s' "$vout" | grep -qF "| W904 |" && pass "W904 for the unresolved ref" || fail "W904 missing"
    printf '%s' "$vout" | grep -qF "| E115 |" && fail "E115 duplicates W904" || pass "no E115 alongside W904"

    SCENARIO_NAME="fmea report lists a duplicate-id row once and keeps rows without S/O/D"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    rep=$("$SYSCRIBE" -m "$Q" fmea report 2>/dev/null || true)
    n=$(printf '%s' "$rep" | grep -c "^| FM-QUAL-001 |" || true)
    [ "$n" -eq 1 ] && pass "duplicate FM-QUAL-001 listed once" || fail "FM-QUAL-001 listed $n times"
    printf '%s' "$rep" | grep -q "^| FM-QUAL-002 |" && pass "row with no S/O/D is listed" || fail "FM-QUAL-002 missing"

    SCENARIO_NAME="fault-tree render FT-KERN-001 emits Mermaid flowchart"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    mermaid=$("$SYSCRIBE" -m "$M" fault-tree render FT-KERN-001 2>/dev/null || true)
    printf '%s' "$mermaid" | head -1 | grep -qF "flowchart TD" \
        && pass "fault-tree render starts with flowchart TD" \
        || fail "fault-tree render missing flowchart TD header"
    printf '%s' "$mermaid" | grep -qF "FTE-PWR-001" \
        && pass "fault-tree render contains FTE-PWR-001 node" \
        || fail "fault-tree render missing FTE-PWR-001 node"
}
