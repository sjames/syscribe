tc_TRS_FM_004() {
    local F="$1"
    local M="$F/TC-TRS-FM-004/mand"
    local L="$F/TC-TRS-FM-004/legacy"

    SCENARIO_NAME="mandatory:true alternative parent is core and model is sound"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$M" feature-check --deep 2>/dev/null || true)
    SCENARIO_OUTPUT="$out"
    # model is sound: no void / no invalid-config codes
    assert_no_code "E223"
    assert_no_code "E225"
    # the parent appears on the "core features:" line (mandatory must make it forced)
    local core; core=$(printf '%s' "$out" | grep -F "core features:" | head -1 || true)
    printf '%s' "$core" | grep -qF "Features::Drive" \
        && pass "mandatory parent on core features line" || fail "Features::Drive not on core features line"

    SCENARIO_NAME="legacy groupKind: mandatory child still treated as forced"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local lo; lo=$("$SYSCRIBE" -m "$L" feature-check --deep 2>/dev/null || true)
    local lcore; lcore=$(printf '%s' "$lo" | grep -F "core features:" | head -1 || true)
    printf '%s' "$lcore" | grep -qF "Features::Power::Ecu" \
        && pass "legacy mandatory child on core features line" || fail "legacy Ecu not on core features line"
}
