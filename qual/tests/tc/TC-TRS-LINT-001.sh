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

    SCENARIO_NAME="a nonexistent path is a usage error (issue #130)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local o e rc
    for args in "$DOCS/nonexist.md" "$DOCS/valid-ref.md $DOCS/nonexist.md"; do
        # shellcheck disable=SC2086
        o=$("$SYSCRIBE" -m "$M" lint-docs $args 2>/dev/null) && rc=0 || rc=$?
        # shellcheck disable=SC2086
        e=$("$SYSCRIBE" -m "$M" lint-docs $args 2>&1 >/dev/null || true)
        [ "$rc" -eq 1 ] && [ -z "$o" ] && grep -qF "nonexist.md" <<<"$e" && grep -qF "does not exist" <<<"$e" \
            && pass "lint-docs ${args##*/} → exit 1, names the missing path" \
            || fail "lint-docs ${args##*/} → exit $rc, stdout ${#o} bytes, stderr: $(head -c 160 <<<"$e")"
    done

    SCENARIO_NAME="--deny takes a code, not a path, and makes W103 gating (issue #130)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    e=$("$SYSCRIBE" -m "$M" lint-docs "$DOCS/valid-ref.md" --deny W099 2>&1) && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && ! grep -qF "W099' does not exist" <<<"$e" \
        && pass "--deny W099 is a code (not scanned as a path), exit 0" || fail "--deny W099 → exit $rc: $(head -c 160 <<<"$e")"
    local PKG="$F/TC-TRS-PKG-002/model"
    "$SYSCRIBE" -m "$PKG" lint-docs "$PKG/Enum/_index.md" >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && pass "W103 alone is advisory (exit 0)" || fail "W103 alone → exit $rc"
    o=$("$SYSCRIBE" -m "$PKG" lint-docs "$PKG/Enum/_index.md" --deny W103 2>/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && grep -qF "W103" <<<"$o" && pass "--deny W103 → W103 reported, exit 1" || fail "--deny W103 → exit $rc"
    "$SYSCRIBE" -m "$PKG" lint-docs "$PKG/Enum/_index.md" --deny=W099,W103 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "--deny=W099,W103 → exit 1" || fail "--deny=W099,W103 → exit $rc"
    for args in "--deny W999" "--bogus"; do
        # shellcheck disable=SC2086
        o=$("$SYSCRIBE" -m "$M" lint-docs "$DOCS/valid-ref.md" $args 2>/dev/null) && rc=0 || rc=$?
        [ "$rc" -eq 1 ] && [ -z "$o" ] && pass "lint-docs $args → usage error, exit 1" || fail "lint-docs $args → exit $rc"
    done
}
