tc_TRS_SYSMLV2_019() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-019/model"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out mx refs
    # edge <from> <to>: is the allocation edge in `matrix --allocations --json` (short names)?
    edge() { jq -e --arg f "$1" --arg t "$2" 'any(.cells[]; .source == $f and .target == $t)' <<<"$mx" >/dev/null 2>&1; }
    # ref <target qname> <allocation qname> <field>: does `refs` list the lifted link?
    ref() {
        refs=$("$SYSCRIBE" -m "$M" refs "$1" 2>&1) || true
        grep -qE "^\| $2 \| $3 \| Allocation \|" <<<"$refs"
    }

    mx=$("$SYSCRIBE" -m "$M" matrix --allocations --json 2>/dev/null) || true
    out=$("$SYSCRIBE" -m "$M" validate 2>/dev/null) || true

    _scn "a cross-package allocate clause becomes allocatedFrom/allocatedTo"
    ref "Arch::SwPackage" "SysML2::Deploy::deploy" "allocatedFrom" \
        && pass "deploy allocatedFrom Arch::SwPackage" || fail "allocatedFrom not lifted/resolved: $refs"
    ref "Arch::Board" "SysML2::Deploy::deploy" "allocatedTo" \
        && pass "deploy allocatedTo Arch::Board" || fail "allocatedTo not lifted/resolved: $refs"

    _scn "the ingested allocation feeds the unified allocation set"
    grep -F "| E314 |" <<<"$out" | grep -qF "SwPackage" \
        && fail "E314 raised although SwPackage is allocated to hardware: $out" \
        || pass "no E314 for SwPackage"
    edge "SwPackage" "Board" && pass "edge SwPackage -> Board in matrix --allocations" \
        || fail "edge missing from matrix --allocations: $mx"

    _scn "feature chains resolve through the heads' part-def types"
    ref "SysML2::Deploy::Controller::ctl" "SysML2::Deploy::chained" "allocatedFrom" \
        && pass "sys.ctl resolved to SysML2::Deploy::Controller::ctl" || fail "sys.ctl not resolved: $refs"
    ref "SysML2::Deploy::Hw::mcu" "SysML2::Deploy::chained" "allocatedTo" \
        && pass "board.mcu resolved to SysML2::Deploy::Hw::mcu" || fail "board.mcu not resolved: $refs"

    _scn "an unresolvable chain tail is truncated with W542"
    grep -F "| W542 |" <<<"$out" | grep -qF "'sys.nosuch'" \
        && pass "W542 names sys.nosuch" || fail "no W542 for sys.nosuch: $out"
    ref "SysML2::Deploy::sys" "SysML2::Deploy::truncated" "allocatedFrom" \
        && pass "truncated to the head SysML2::Deploy::sys" || fail "not truncated to the head: $refs"

    _scn "endpoints that resolve nowhere are reported"
    grep -F "| E502 |" <<<"$out" | grep -qF "'NoSuchSource'" \
        && pass "E502 names NoSuchSource" || fail "no E502 for NoSuchSource: $out"
    grep -F "| E503 |" <<<"$out" | grep -qF "'NoSuchTarget'" \
        && pass "E503 names NoSuchTarget" || fail "no E503 for NoSuchTarget: $out"
}
