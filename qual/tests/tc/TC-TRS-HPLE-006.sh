tc_TRS_HPLE_006() {
    local F="$1"; local B="$F/TC-TRS-HPLE-006"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local pack top fc rows

    pack=$("$SYSCRIBE" -m "$B/pack/model" validate 2>/dev/null) || true
    top=$("$SYSCRIBE" -m "$B/top/model" validate 2>/dev/null) || true

    _scn "a mount-path subConfigurations entry and binding key resolve"
    grep -qF "| E516 |" <<<"$pack" && fail "E516 for the mount-path subConfigurations entry: $pack" \
        || pass "Vendor::CellConfs::CONF-CELL-001 resolves (no E516)"
    grep -qF "| E222 |" <<<"$pack" && fail "E222 for the mount-path binding key: $pack" \
        || pass "Vendor::CellFeatures::Cell.capacityAh resolves (no E222)"

    _scn "the mount-path binding closes the parameter"
    rows=$(grep -F "| W513 |" <<<"$pack" || true)
    grep -qF "'Features::Cell.siteCode'" <<<"$rows" && pass "W513 reports siteCode open" \
        || fail "W513 missing for the genuinely open siteCode: $pack"
    grep -qF "capacityAh" <<<"$rows" && fail "capacityAh still reported open: $rows" \
        || pass "capacityAh closed by the mount-path binding"

    _scn "feature-check resolves the mount-path binding key too"
    fc=$("$SYSCRIBE" -m "$B/pack/model" feature-check 2>&1) || true
    grep -qF "E222" <<<"$fc" && fail "feature-check raised E222: $fc" || pass "feature-check: no E222"

    _scn "a top tier consolidating through a mount path validates cleanly"
    rows=$(grep -F "CONF-TOP-OK-001.md" <<<"$top" | grep -E "\| (E516|E518|E222|E523|W513) \|" || true)
    [ -z "$rows" ] && pass "CONF-TOP-OK-001 raises no E516/E518/E222/E523/W513" \
        || fail "unexpected findings on CONF-TOP-OK-001: $rows"

    _scn "a nearer tier's mount-path binding counts as already closing the parameter"
    rows=$(grep -F "| E523 |" <<<"$top" | grep -F "CONF-TOP-DOUBLE-001.md" || true)
    grep -qF "'Features::Cell.capacityAh'" <<<"$rows" && grep -qF "CONF-PACK-001" <<<"$rows" \
        && pass "E523 names capacityAh already bound by CONF-PACK-001" \
        || fail "no E523 for the re-bound capacityAh: $top"

    _scn "a mount path naming nothing in its peer is dangling"
    rows=$(grep -F "| E516 |" <<<"$top" | grep -F "CONF-TOP-BAD-001.md" || true)
    grep -qF "'Supply::PackConfs::CONF-NOPE-001'" <<<"$rows" && grep -qF "repo 'pack'" <<<"$rows" \
        && pass "E516 names the dangling mount path in repo 'pack'" \
        || fail "no E516 for the dangling mount path: $top"
}
