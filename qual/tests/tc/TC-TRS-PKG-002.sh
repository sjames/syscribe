tc_TRS_PKG_002() {
    local F="$1"; local B="$F/TC-TRS-PKG-002"; local M="$B/model" out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "an _index.md enumerating three members raises W103 and exits zero"
    out=$("$SYSCRIBE" -m "$M" lint-docs "$M/Enum/_index.md" 2>&1) && pass "exit zero" || fail "non-zero exit: $out"
    printf '%s' "$out" | grep -q 'W103' && pass "W103 raised" || fail "no W103: $out"
    printf '%s' "$out" | grep 'W103' | grep -q 'enumerates 3 of' && pass "count named" || fail "count not named"

    _scn "two members, foreign ids, or a non-_index.md file raise nothing"
    out=$("$SYSCRIBE" -m "$M" lint-docs "$M/Two/_index.md" "$M/Foreign/_index.md" "$B/docs/notes.md" 2>&1 || true)
    printf '%s' "$out" | grep -q 'W103' && fail "unexpected W103: $out" || pass "no W103"
    out=$("$SYSCRIBE" -m "$M" lint-docs "$M" 2>&1 || true)
    [ "$(printf '%s' "$out" | grep -c 'W103')" = "1" ] && pass "directory scan flags only Enum" || fail "directory scan wrong: $out"
}
