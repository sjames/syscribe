tc_TRS_LINT_001() {
    local F="$1"; local M="$F/TC-TRS-LINT-001/model"
    local DOCS="$F/TC-TRS-LINT-001/docs"

    SCENARIO_NAME="resolvable stable ID token produces no output and exits 0"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" lint-docs "$DOCS/valid-ref.md" 2>/dev/null; echo "EXIT:$?")
    printf '%s' "$out" | grep -qF "EXIT:0" \
        && pass "exit 0 for file with valid refs" || fail "non-zero exit for file with valid refs"
    printf '%s' "$out" | grep -vF "EXIT:" | grep -qF "W099" \
        && fail "W099 wrongly emitted for resolvable token" || pass "no W099 for resolvable token"

    SCENARIO_NAME="unresolvable stable ID token causes W099 and exits 1"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" lint-docs "$DOCS/invalid-ref.md" 2>/dev/null; echo "EXIT:$?")
    printf '%s' "$out" | grep -qF "W099" \
        && pass "W099 emitted for unresolvable token" || fail "W099 not emitted for unresolvable token"
    printf '%s' "$out" | grep -qF "REQ-TRS-NONEXIST-001" \
        && pass "W099 names the unresolvable token" || fail "W099 missing token name"
    printf '%s' "$out" | grep -qF "EXIT:1" \
        && pass "exit 1 for unresolvable token" || fail "exit 0 for unresolvable token (should be 1)"

    SCENARIO_NAME="directory scan finds only invalid references"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    w099_count=$("$SYSCRIBE" -m "$M" lint-docs "$DOCS/" 2>/dev/null | grep -c "W099" || true)
    [ "$w099_count" -eq 1 ] \
        && pass "directory scan found exactly 1 W099 finding" \
        || fail "directory scan found $w099_count W099 findings (expected 1)"

    SCENARIO_NAME="--json emits JSON array with expected fields"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    json=$("$SYSCRIBE" -m "$M" lint-docs "$DOCS/invalid-ref.md" --json 2>/dev/null || true)
    printf '%s' "$json" | python3 -c "
import json, sys
items = json.load(sys.stdin)
assert isinstance(items, list), 'not a list'
assert len(items) == 1, f'expected 1 finding, got {len(items)}'
f = items[0]
assert f['code'] == 'W099', f'code is {f[\"code\"]}'
assert f['token'] == 'REQ-TRS-NONEXIST-001', f'token is {f[\"token\"]}'
assert 'file' in f, 'missing file'
assert 'line' in f, 'missing line'
print('ok')
" 2>/dev/null && pass "lint-docs --json emits correct finding object" || fail "lint-docs --json has wrong structure"

    SCENARIO_NAME="file with no stable-ID tokens exits 0 and produces no output"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    tmp=$(mktemp /tmp/lint-docs-XXXXX.md)
    printf '# Plain doc\n\nNo IDs here, just text.\n' > "$tmp"
    out=$("$SYSCRIBE" -m "$M" lint-docs "$tmp" 2>/dev/null; echo "EXIT:$?")
    rm -f "$tmp"
    printf '%s' "$out" | grep -qF "EXIT:0" \
        && pass "exit 0 for file with no stable-ID tokens" \
        || fail "non-zero exit for file with no stable-ID tokens"
}
