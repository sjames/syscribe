tc_TRS_PKG_001() {
    local F="$1"; local M="$F/TC-TRS-PKG-001/model" out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "show lists direct members from the directory"
    out=$("$SYSCRIBE" -m "$M" show Reqs 2>&1 || true)
    printf '%s' "$out" | grep -q '^## Members (4)' && pass "Members (4) heading" || fail "no '## Members (4)' heading: $out"
    for id in REQ-PK-001 REQ-PK-002 REQ-PK-003 Reqs::Sub; do
        printf '%s' "$out" | sed -n '/^## Members/,$p' | grep -qF -- "$id" && pass "lists $id" || fail "missing $id"
    done
    printf '%s' "$out" | sed -n '/^## Members/,$p' | grep -q 'REQ-PK-010' && fail "grandchild listed" || pass "grandchild not listed"
    printf '%s' "$out" | sed -n '/^## Members/,$p' | grep 'REQ-PK-001' | grep -q 'approved' && pass "status shown" || fail "status missing"
    printf '%s' "$out" | sed -n '/^## Members/,$p' | grep 'REQ-PK-002' | grep -q 'Requirement' && pass "type shown" || fail "type missing"

    _scn "--no-related keeps the member list"
    out=$("$SYSCRIBE" -m "$M" show Reqs --no-related 2>&1 || true)
    printf '%s' "$out" | grep -q '^## Members (4)' && pass "members kept" || fail "members dropped"

    _scn "an empty package shows an explicit empty state"
    out=$("$SYSCRIBE" -m "$M" show Empty 2>&1 || true)
    printf '%s' "$out" | grep -q '^## Members (0)' && pass "empty state" || fail "no empty state: $out"

    _scn "export-html package page lists members with links"
    local tmp; tmp=$(mktemp -d)
    "$SYSCRIBE" -m "$M" export-html --out "$tmp/site" >/dev/null 2>&1 || true
    local page sect; page="$tmp/site/Reqs.html"
    [ -f "$page" ] || page=$(ls "$tmp"/site/*Reqs.html "$tmp"/site/*/Reqs.html 2>/dev/null | head -1 || true)
    sect=$(sed -n '/id="members"/,/<\/section>/p' "$page" 2>/dev/null || true)
    printf '%s' "$sect" | grep -q 'REQ-PK-003' && pass "members section lists REQ-PK-003" || fail "no members section listing REQ-PK-003 in $page"
    printf '%s' "$sect" | grep -q 'href="[^"]*REQ-PK-003[^"]*"' && pass "member linked" || fail "member not linked"
    printf '%s' "$sect" | grep -q 'REQ-PK-010' && fail "grandchild in members section" || pass "grandchild excluded"
    rm -rf "$tmp"
}
