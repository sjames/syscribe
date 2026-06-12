tc_TRS_SCRIPT_005() {
    local F="$1"
    local M="$F/TC-TRS-SCRIPT-005"
    local NONE; NONE=$(mktemp -d)
    printf '[scripts]\npath = ".syscribe/scripts"\n' > "$NONE/.syscribe.toml"
    printf -- '---\ntype: Package\nname: Empty\n---\nEmpty.\n' > "$NONE/_index.md"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "list shows a command and a check with kind/description/source"
    local lst rc=0
    lst=$("$SYSCRIBE" -m "$M" scripts list 2>/dev/null) || rc=$?
    [ "$rc" -eq 0 ] && pass "scripts list exit 0" || fail "scripts list exit $rc"
    printf '%s' "$lst" | grep -qF "coverage" \
        && pass "command name shown" || fail "command name missing"
    printf '%s' "$lst" | grep -qE "\| coverage .* command " \
        && pass "kind command shown" || fail "kind command missing"
    printf '%s' "$lst" | grep -qE "\| naming .* check " \
        && pass "kind check shown" || fail "kind check missing"
    printf '%s' "$lst" | grep -qiF "Summarise requirement coverage" \
        && pass "description shown" || fail "description missing"
    printf '%s' "$lst" | grep -qiF "coverage.rhai" \
        && pass "source file shown" || fail "source file missing"

    _scn "list --json carries the same fields"
    local js; js=$("$SYSCRIBE" -m "$M" scripts list --json 2>/dev/null) || true
    printf '%s' "$js" | grep -qF '"name"' && pass "json has name" || fail "json missing name"
    printf '%s' "$js" | grep -qF '"kind"' && pass "json has kind" || fail "json missing kind"
    printf '%s' "$js" | grep -qF '"description"' && pass "json has description" || fail "json missing description"
    printf '%s' "$js" | grep -qiF 'coverage.rhai' && pass "json has source" || fail "json missing source"

    _scn "run a command prints its returned string"
    local out; rc=0; out=$("$SYSCRIBE" -m "$M" scripts run coverage 2>/dev/null) || rc=$?
    [ "$rc" -eq 0 ] && pass "scripts run coverage exit 0" || fail "exit $rc"
    printf '%s' "$out" | grep -qF "coverage: 2 requirements" \
        && pass "returned string printed" || fail "returned string wrong: $out"

    _scn "unknown command exits non-zero"
    rc=0; out=$("$SYSCRIBE" -m "$M" scripts run nope 2>&1) || rc=$?
    [ "$rc" -ne 0 ] && pass "unknown command non-zero" || fail "unknown command exited 0"
    printf '%s' "$out" | grep -qiF "nope" && pass "names the unknown command" || fail "no message for unknown"

    _scn "a check is not runnable via scripts run"
    rc=0; out=$("$SYSCRIBE" -m "$M" scripts run naming 2>&1) || rc=$?
    [ "$rc" -ne 0 ] && pass "running a check via run exits non-zero" || fail "check ran as command"
    printf '%s' "$out" | grep -qiF "check" \
        && pass "reports that it is a check" || fail "did not say it is a check"

    _scn "no scripts directory reports none and exits 0"
    rc=0; out=$("$SYSCRIBE" -m "$NONE" scripts list 2>/dev/null) || rc=$?
    [ "$rc" -eq 0 ] && pass "exit 0 with no scripts dir" || fail "exit $rc"
    printf '%s' "$out" | grep -qiF "no" && pass "reports none defined" || fail "did not report none"

    rm -rf "$NONE"
}
