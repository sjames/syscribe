tc_TRS_LINKTYPE_007() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-007"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local M="$FX/model" out
    _scn "one hop forward"
    out=$("$SYSCRIBE" -m "$M" follow REQ-LT7-001 mitigates 2>&1) || true
    printf '%s' "$out" | grep -q 'REQ-LT7-002' && ! printf '%s' "$out" | grep -q 'REQ-LT7-003' && pass "one hop reaches only 002" || fail "one hop wrong: $out"
    _scn "transitive forward"
    out=$("$SYSCRIBE" -m "$M" follow REQ-LT7-001 mitigates --transitive 2>&1) || true
    printf '%s' "$out" | grep -q 'REQ-LT7-003' && pass "transitive reaches 003" || fail "transitive missed 003"
    _scn "depth bounds the traversal"
    out=$("$SYSCRIBE" -m "$M" follow REQ-LT7-001 mitigates --depth 1 2>&1) || true
    printf '%s' "$out" | grep -q 'REQ-LT7-003' && fail "depth 1 reached 003" || pass "depth 1 bounded"
    printf '%s' "$out" | grep -q 'REQ-LT7-002' && pass "depth 1 reaches 002" || fail "depth 1 missed 002"
    out=$("$SYSCRIBE" -m "$M" follow REQ-LT7-001 mitigates --depth 2 2>&1) || true
    printf '%s' "$out" | grep -q 'REQ-LT7-003' && pass "depth 2 reaches 003 (depth implies transitive)" || fail "depth 2 missed 003"
    _scn "inverse name and --reverse traverse backwards"
    out=$("$SYSCRIBE" -m "$M" follow REQ-LT7-003 mitigatedBy 2>&1) || true
    printf '%s' "$out" | grep -q 'REQ-LT7-002' && pass "inverse reaches 002" || fail "inverse wrong"
    out=$("$SYSCRIBE" -m "$M" follow REQ-LT7-003 mitigates --reverse --transitive 2>&1) || true
    printf '%s' "$out" | grep -q 'REQ-LT7-001' && pass "--reverse --transitive reaches 001" || fail "--reverse wrong"
    _scn "transitive traversal terminates on cycles"
    out=$(timeout 10 "$SYSCRIBE" -m "$M" follow REQ-LT7-010 loops --transitive 2>&1) && pass "cycle terminates" || fail "cycle did not terminate"
    printf '%s' "$out" | grep -q 'REQ-LT7-011' && pass "cycle member reached" || fail "cycle member missing"
    _scn "built-in link and reverse-index names"
    out=$("$SYSCRIBE" -m "$M" follow TC-LT7-001 verifies 2>&1) || true
    printf '%s' "$out" | grep -q 'REQ-LT7-001' && pass "built-in verifies" || fail "built-in verifies wrong"
    out=$("$SYSCRIBE" -m "$M" follow REQ-LT7-001 verifiedBy 2>&1) || true
    printf '%s' "$out" | grep -q 'TC-LT7-001' && pass "built-in verifiedBy" || fail "built-in verifiedBy wrong"
    _scn "json and dot formats"
    out=$("$SYSCRIBE" -m "$M" follow REQ-LT7-001 mitigates --transitive --format json 2>/dev/null) || true
    printf '%s' "$out" | jq -e '.link=="mitigates" and .direction=="forward" and (.results|length)==2 and (.results[]|select(.id=="REQ-LT7-003")|.depth)==2' >/dev/null \
        && pass "json complete" || fail "json wrong: $out"
    out=$("$SYSCRIBE" -m "$M" follow REQ-LT7-001 mitigates --format dot 2>/dev/null) || true
    printf '%s' "$out" | grep -q 'digraph' && pass "dot output" || fail "no dot output"
    _scn "unknown link or element exits non-zero"
    out=$("$SYSCRIBE" -m "$M" follow REQ-LT7-001 nosuchlink 2>&1) && fail "unknown link exit 0" || pass "unknown link non-zero"
    printf '%s' "$out" | grep -q 'mitigates' && pass "available names listed" || fail "names not listed"
    "$SYSCRIBE" -m "$M" follow REQ-NOPE-001 mitigates >/dev/null 2>&1 && fail "unknown element exit 0" || pass "unknown element non-zero"
}
