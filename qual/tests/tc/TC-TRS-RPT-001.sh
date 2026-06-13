tc_TRS_RPT_001() {
    local F="$1"; local M="$F/TC-TRS-RPT-001/model"

    SCENARIO_NAME="fmea report prints Markdown table with correct column headers"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" fmea report 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "| ID | Name | Failure Mode | Effect | Severity | Occurrence | Detection | RPN | Controls | Status |" \
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

    SCENARIO_NAME="fault-tree render FT-KERN-001 emits Mermaid flowchart"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    mermaid=$("$SYSCRIBE" -m "$M" fault-tree render FT-KERN-001 2>/dev/null || true)
    printf '%s' "$mermaid" | head -1 | grep -qF "flowchart TD" \
        && pass "fault-tree render starts with flowchart TD" \
        || fail "fault-tree render missing flowchart TD header"
    printf '%s' "$mermaid" | grep -qF "FTE-PWR-001" \
        && pass "fault-tree render contains FTE-PWR-001 node" \
        || fail "fault-tree render missing FTE-PWR-001 node"
}
