tc_TRS_LINKTYPE_011() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-011"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local tmp out; tmp=$(mktemp -d); cp -r "$FX/model" "$tmp/m"; local M="$tmp/m"
    _scn "suspect list lists unbaselined custom links except opted-out types"
    out=$("$SYSCRIBE" -m "$M" suspect list 2>&1 || true)
    printf '%s' "$out" | grep -q 'REQ-LT11-001' && pass "custom link listed" || fail "custom link not listed: $out"
    printf '%s' "$out" | grep -q 'REQ-LT11-002' && fail "opted-out link listed" || pass "opted-out link not listed"
    _scn "a baselined custom link raises W090 after its target changes"
    "$SYSCRIBE" -m "$M" suspect accept --all-unbaselined >/dev/null 2>&1 || true
    grep -q 'REQ-LT11-001' "$M/Arch/Ctl.md" && pass "baseline stored" || fail "baseline not stored"
    sed -i 's/ORIGINAL/CHANGED/' "$M/Requirements/REQ-LT11-001.md" "$M/Requirements/REQ-LT11-002.md"
    out=$("$SYSCRIBE" -m "$M" validate 2>&1 || true)
    printf '%s' "$out" | grep 'W090' | grep -q 'REQ-LT11-001' && pass "W090 raised" || fail "W090 not raised"
    _scn "an opted-out type never raises W090"
    printf '%s' "$out" | grep 'W090' | grep -q 'REQ-LT11-002' && fail "W090 on opted-out type" || pass "opted-out silent"
    rm -rf "$tmp"
}
