tc_TRS_SYSMLV2_018() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-018/model"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out show e111

    _scn "a package-level allocation def maps to AllocationDef"
    show=$("$SYSCRIBE" -m "$M" show SysML2::Deploy::SoftwareToHardware 2>&1) || true
    grep -qF "AllocationDef" <<<"$show" && pass "SoftwareToHardware is an AllocationDef" \
        || fail "SoftwareToHardware is not an AllocationDef: $show"
    grep -qF "Deploys a software component onto a hardware board." <<<"$show" \
        && pass "doc text lifted" || fail "doc text not lifted: $show"

    _scn "an allocation def nested in a part def maps to AllocationDef"
    show=$("$SYSCRIBE" -m "$M" show SysML2::Deploy::Rack::RackSlot 2>&1) || true
    grep -qF "AllocationDef" <<<"$show" && pass "Rack::RackSlot is an AllocationDef" \
        || fail "Rack::RackSlot is not an AllocationDef: $show"

    out=$("$SYSCRIBE" -m "$M" validate 2>/dev/null) || true
    e111=$(grep -F "| E111 |" <<<"$out" || true)

    _scn "allocation usages typed by an ingested allocation def resolve"
    grep -qF "'SoftwareToHardware'" <<<"$e111" && fail "E111 for SoftwareToHardware: $e111" \
        || pass "deployCtl's typedBy resolves"
    grep -qF "'Rack::RackSlot'" <<<"$e111" && fail "E111 for Rack::RackSlot: $e111" \
        || pass "slotUse's typedBy resolves"

    _scn "an allocation usage typed by an unknown name raises E111"
    grep -qF "'NoSuchAllocationDef'" <<<"$e111" && pass "E111 names NoSuchAllocationDef" \
        || fail "no E111 for the unresolvable allocation typedBy: $e111"
}
