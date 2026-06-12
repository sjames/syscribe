tc_TRS_OUT_014() {
    local F="$1"; local M="$F/TC-TRS-OUT-014/model"

    SCENARIO_NAME="list TestCase table has execution-oriented columns (ID/Name/Level/Status/Verifies/Tags)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" list TestCase 2>/dev/null)
    printf '%s' "$out" | grep -qF "| ID | Name | Level | Status | Verifies | Tags |" \
        && pass "TestCase table has correct headers" || fail "TestCase table missing expected headers"

    SCENARIO_NAME="table row shows TC id, level, verifies, and tags"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qF "TC-BOOT-001" \
        && pass "TC-BOOT-001 row present" || fail "TC-BOOT-001 missing from table"
    printf '%s' "$out" | grep "TC-BOOT-001" | grep -qF "REQ-BOOT-001" \
        && pass "verifies column shows REQ-BOOT-001" || fail "verifies column missing REQ-BOOT-001"
    printf '%s' "$out" | grep "TC-BOOT-001" | grep -qF "integration" \
        && pass "tags column shows integration tag" || fail "tags column missing integration tag"

    SCENARIO_NAME="list TestCase --json includes testLevel, verifies, tags, sourceFile, testFunctions"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    json=$("$SYSCRIBE" -m "$M" list TestCase --json 2>/dev/null)
    printf '%s' "$json" | python3 -c "
import json, sys
items = json.load(sys.stdin)
tc = next(x for x in items if x.get('id') == 'TC-BOOT-001')
assert 'testLevel' in tc, 'missing testLevel'
assert 'verifies' in tc, 'missing verifies'
assert 'tags' in tc, 'missing tags'
assert 'sourceFile' in tc, 'missing sourceFile'
assert 'testFunctions' in tc, 'missing testFunctions'
assert tc['testLevel'] == 'L3', f'wrong testLevel: {tc[\"testLevel\"]}'
assert 'REQ-BOOT-001' in tc['verifies'], 'REQ-BOOT-001 not in verifies'
assert 'integration' in tc['tags'], 'integration not in tags'
assert tc['sourceFile'] == 'tests/boot_tests.rs', f'wrong sourceFile: {tc[\"sourceFile\"]}'
assert len(tc['testFunctions']) == 1, f'wrong testFunctions count: {len(tc[\"testFunctions\"])}'
print('ok')
" 2>/dev/null && pass "JSON fields correct for TC-BOOT-001" || fail "JSON fields missing or wrong for TC-BOOT-001"

    SCENARIO_NAME="list TestCase --config CONF-WITH-X projects to only active TCs"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" list TestCase --config CONF-WITH-X 2>/dev/null)
    printf '%s' "$out" | grep -qF "TC-BOOT-002" \
        && pass "TC-BOOT-002 active in CONF-WITH-X" || fail "TC-BOOT-002 missing under CONF-WITH-X projection"

    SCENARIO_NAME="list TestCase --config CONF-WITH-X --tag integration narrows further"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    ids=$(printf '%s' "$("$SYSCRIBE" -m "$M" list TestCase --config CONF-WITH-X --tag integration 2>/dev/null)" \
        | grep -oE 'TC-BOOT-[0-9]+' | sort | paste -sd,)
    [ "$ids" = "TC-BOOT-001,TC-BOOT-002" ] \
        && pass "config+tag returns both integration TCs" || fail "got '$ids' (expected TC-BOOT-001,TC-BOOT-002)"

    SCENARIO_NAME="list Requirement still uses the generic table (no regression)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" list Requirement 2>/dev/null)
    printf '%s' "$out" | grep -qF "| Qualified Name |" \
        && pass "Requirement table still uses generic headers" || fail "Requirement table unexpectedly changed"
}
