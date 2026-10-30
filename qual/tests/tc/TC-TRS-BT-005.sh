tc_TRS_BT_005() {
    local F="$1"; local B="$F/TC-TRS-BT-002"
    local out a b
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "CLI and JSON"
    a=$("$SYSCRIBE" -m "$B/deploy" binding-times --json 2>/dev/null) || true
    b=$("$SYSCRIBE" -m "$B/deploy" binding-times --json 2>/dev/null) || true
    [ -n "$a" ] && [ "$a" = "$b" ] && pass "JSON output is identical across runs" || fail "JSON differs or is empty"
    printf '%s' "$a" | jq -e '[.bindingTimes[].order]==[0,1,2,3]' >/dev/null \
        && pass "ordered earliest to latest" || fail "order field wrong: $a"
    out=$("$SYSCRIBE" -m "$B/deploy" binding-times 2>&1) || true
    printf '%s' "$out" | grep -q 'deploy' && printf '%s' "$out" | grep -q 'installer' \
        && pass "text lists the stage and its binder" || fail "text incomplete: $out"

    _scn "agent instructions"
    out=$("$SYSCRIBE" -m "$B/deploy" --agent-instructions 2>&1) || true
    printf '%s' "$out" | grep -q '## Project binding times' && printf '%s' "$out" | grep -q '`deploy`' \
        && pass "the section lists the declared stage" || fail "section missing"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-BT-001/plain" --agent-instructions 2>&1) || true
    printf '%s' "$out" | grep -q '## Project binding times' && fail "section appended for the built-in vocabulary" \
        || pass "nothing appended for the built-in vocabulary"
}
