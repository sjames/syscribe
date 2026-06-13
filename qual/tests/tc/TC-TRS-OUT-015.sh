tc_TRS_OUT_015() {
    local F="$1"; local M="$F/TC-TRS-OUT-015/model"

    SCENARIO_NAME="list AssumptionOfUse prints SRAC-oriented table headers"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" list AssumptionOfUse 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "| ID | Name | Applies To | Status |" \
        && pass "AssumptionOfUse table has correct headers" \
        || fail "AssumptionOfUse table missing expected headers"

    SCENARIO_NAME="AOU-SYS-001 row shows both SG refs in Applies To column"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep "AOU-SYS-001" | grep -qF "SG-CTRL-001" \
        && pass "AOU-SYS-001 row shows SG-CTRL-001 in Applies To" \
        || fail "AOU-SYS-001 row missing SG-CTRL-001 in Applies To"

    SCENARIO_NAME="AOU-SYS-002 with no appliesTo shows dash in Applies To column"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep "AOU-SYS-002" | grep -qE "\| — \|" \
        && pass "AOU-SYS-002 shows dash for empty appliesTo" \
        || fail "AOU-SYS-002 does not show dash for empty appliesTo"

    SCENARIO_NAME="list AssumptionOfUse --json includes appliesTo array and body string"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    json=$("$SYSCRIBE" -m "$M" list AssumptionOfUse --json 2>/dev/null || true)
    printf '%s' "$json" | python3 -c "
import json, sys
items = json.load(sys.stdin)
a = next(x for x in items if x.get('id') == 'AOU-SYS-001')
assert 'appliesTo' in a, 'missing appliesTo'
assert isinstance(a['appliesTo'], list), 'appliesTo not a list'
assert 'SG-CTRL-001' in a['appliesTo'], 'SG-CTRL-001 not in appliesTo'
assert 'body' in a, 'missing body'
assert a['body'] is not None, 'body is null but should have content'
b = next(x for x in items if x.get('id') == 'AOU-SYS-002')
assert b.get('body') is None, 'AOU-SYS-002 body should be null'
print('ok')
" 2>/dev/null && pass "JSON fields correct for AOU elements" || fail "JSON fields missing or wrong for AOU elements"

    SCENARIO_NAME="list Requirement still uses generic table (no regression)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    req_out=$("$SYSCRIBE" -m "$M" list Requirement 2>/dev/null || true)
    printf '%s' "$req_out" | grep -qF "Applies To" \
        && fail "list Requirement table unexpectedly shows Applies To column" \
        || pass "list Requirement table uses generic headers (no regression)"
}
