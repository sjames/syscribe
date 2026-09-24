#!/usr/bin/env bash
# qual/tests/lib.sh — assertion library for syscribe tool qualification tests
#
# Globals expected from caller:
#   SYSCRIBE     path to syscribe binary
#   RESULTS_FILE path to NDJSON results file

TC_ID=""; TC_TITLE=""; TC_FAILED=0; PASS_COUNT=0; FAIL_COUNT=0
_RESULT_LINES=""
SCENARIO_OUTPUT=""; SCENARIO_EXIT=0; SCENARIO_NAME=""
_SCEN_PASS=0; _SCEN_FAIL=0

if [ -t 1 ]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BOLD='\033[1m'; NC='\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; BOLD=''; NC=''
fi

# ── TC lifecycle ──────────────────────────────────────────────────────────────

start_tc() {
    TC_ID="$1"; TC_TITLE="$2"
    TC_FAILED=0; PASS_COUNT=0; FAIL_COUNT=0; _RESULT_LINES=""; SCENARIO_NAME=""
    printf "\n${BOLD}[%s]${NC} %s\n" "$TC_ID" "$TC_TITLE"
}

_flush_scenario() {
    [ -z "$SCENARIO_NAME" ] && return
    local v="pass"; [ "$_SCEN_FAIL" -gt 0 ] && v="fail"
    local e="{\"name\":\"$(printf '%s' "$SCENARIO_NAME" | sed 's/"/\\"/g')\",\"pass\":$_SCEN_PASS,\"fail\":$_SCEN_FAIL,\"verdict\":\"$v\"}"
    [ -z "$_RESULT_LINES" ] && _RESULT_LINES="$e" || _RESULT_LINES="${_RESULT_LINES},$e"
}

end_tc() {
    local verifies="${1:-}"
    _flush_scenario
    local verdict="PASS"; [ "$TC_FAILED" -eq 1 ] && verdict="FAIL"
    [ "$verdict" = "PASS" ] \
        && printf "  ${GREEN}✓ PASS${NC} (%d assertions)\n" "$PASS_COUNT" \
        || printf "  ${RED}✗ FAIL${NC} (%d passed, %d failed)\n" "$PASS_COUNT" "$FAIL_COUNT"
    printf '{"tc_id":"%s","title":"%s","verifies":"%s","pass":%d,"fail":%d,"verdict":"%s","scenarios":[%s]}\n' \
        "$TC_ID" \
        "$(printf '%s' "$TC_TITLE" | sed 's/"/\\"/g')" \
        "$(printf '%s' "$verifies" | sed 's/"/\\"/g')" \
        "$PASS_COUNT" "$FAIL_COUNT" "$verdict" "${_RESULT_LINES}" \
        >> "$RESULTS_FILE"
}

# ── Scenario lifecycle ────────────────────────────────────────────────────────

run_scenario() {
    _flush_scenario
    SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$2" 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
}

# ── Assertions ────────────────────────────────────────────────────────────────

pass() { printf "    ${GREEN}✓${NC} %s\n" "$1"; PASS_COUNT=$((PASS_COUNT+1)); _SCEN_PASS=$((_SCEN_PASS+1)); }
fail() { printf "    ${RED}✗${NC} %s\n" "$1"; FAIL_COUNT=$((FAIL_COUNT+1)); _SCEN_FAIL=$((_SCEN_FAIL+1)); TC_FAILED=1; }

# Helpers feed grep via here-strings, never `printf | grep -q`: under `set -o pipefail`
# a `grep -q` that exits on its first match makes the writer die of SIGPIPE once the
# output exceeds the 64 KiB pipe buffer, failing (or, for negative checks, falsely
# passing) the assertion nondeterministically.
assert_has_code() {
    grep -qF "| $1 |" <<<"$SCENARIO_OUTPUT" \
        && pass "$1 present in output" || fail "$1 not found in output"
}

assert_no_code() {
    grep -qF "| $1 |" <<<"$SCENARIO_OUTPUT" \
        && fail "$1 unexpectedly present in output" || pass "no $1 in output"
}

assert_count() {
    local actual; actual=$(printf '%s' "$SCENARIO_OUTPUT" | grep -cF "| $1 |" || echo 0)
    [ "$actual" -eq "$2" ] && pass "$1 count = $2" || fail "$1 count = $actual (expected $2)"
}

assert_exit_zero() {
    [ "$SCENARIO_EXIT" -eq 0 ] && pass "exit code 0" || fail "exit code $SCENARIO_EXIT (expected 0)"
}

assert_exit_nonzero() {
    [ "$SCENARIO_EXIT" -ne 0 ] && pass "exit code $SCENARIO_EXIT (non-zero)" || fail "exit code 0 (expected non-zero)"
}

assert_output_contains() {
    grep -qF "$1" <<<"$SCENARIO_OUTPUT" \
        && pass "output contains: $1" || fail "output missing: $1"
}

assert_stdout_nonempty() {
    [ -n "$SCENARIO_OUTPUT" ] && pass "stdout non-empty" || fail "stdout empty"
}

# ── Frontmatter helpers ───────────────────────────────────────────────────────

fm_get() {
    awk "/^---/{n++; if(n==2)exit} n==1 && /^${2}:/{gsub(/^${2}:[[:space:]]*/,\"\"); gsub(/\"/,\"\"); print; exit}" "$1"
}

fm_get_list() {
    awk "/^---/{n++; if(n==2)exit} n==1 && /^${2}:/{f=1;next} f && /^  - /{gsub(/^  - /,\"\");print;next} f && /^[^ ]/{f=0}" "$1"
}
