tc_TRS_PLANITEM_005() {
    local F="$1"; local FX="$F/TC-TRS-PLANITEM-005"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)

    # 1. a resolving ref: entry validates cleanly
    _scn "a resolving ref: entry validates cleanly"
    printf '%s' "$out" | grep 'RefValid.md' | grep -q 'E716' \
        && fail "unexpected E716 on RefValid.md" || pass "RefValid.md raises no E716"

    # 2. a dangling ref: entry is rejected
    _scn "a dangling ref: entry is rejected"
    printf '%s' "$out" | grep 'RefDangling.md' | grep -q 'E716' \
        && pass "E716 raised for a dangling ref" || fail "E716 not raised for a dangling ref"

    # 3. the same dangling ref: with a rationale: is waived
    _scn "the same dangling ref: with a rationale: is waived"
    printf '%s' "$out" | grep 'RefWaived.md' | grep -q 'E716' \
        && fail "E716 unexpectedly raised despite rationale" || pass "RefWaived.md raises no E716 (waived)"

    # 4. an existing local path: entry validates cleanly
    _scn "an existing local path: entry validates cleanly"
    printf '%s' "$out" | grep 'PathValid.md' | grep -q 'E717' \
        && fail "unexpected E717 on PathValid.md" || pass "PathValid.md raises no E717"

    # 5. a missing local path: entry is rejected
    _scn "a missing local path: entry is rejected"
    printf '%s' "$out" | grep 'PathMissing.md' | grep -q 'E717' \
        && pass "E717 raised for a missing local path" || fail "E717 not raised for a missing local path"

    # 6. the same missing path: with a rationale: is waived
    _scn "the same missing path: with a rationale: is waived"
    printf '%s' "$out" | grep 'PathWaived.md' | grep -q 'E717' \
        && fail "E717 unexpectedly raised despite rationale" || pass "PathWaived.md raises no E717 (waived)"

    # 7. a remote-URI path: entry skips the local existence check
    _scn "a remote-URI path: entry skips the local existence check"
    printf '%s' "$out" | grep 'PathRemote.md' | grep -q 'E717' \
        && fail "unexpected E717 for a remote URI" || pass "PathRemote.md raises no E717 (remote, unchecked)"

    # 8. a waiver is per-entry, not blanket
    _scn "a waiver is per-entry, not blanket"
    local pe; pe=$(printf '%s' "$out" | grep 'PerEntry.md' || true)
    printf '%s' "$pe" | grep -q 'E717' \
        && fail "the waived path entry was unexpectedly flagged (E717)" \
        || pass "the waived path entry is not flagged"
    printf '%s' "$pe" | grep -q 'E716' \
        && pass "the un-waived ref entry is still flagged (E716)" \
        || fail "the un-waived ref entry was not flagged"
}
