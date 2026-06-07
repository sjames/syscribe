tc_TRS_DISC_003() {
    local F="$1"; local B="$F/TC-TRS-DISC-001/pl"

    SCENARIO_NAME="matrix --features feature × configuration grid"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out ec
    out=$("$SYSCRIBE" -m "$B" matrix --features 2>/dev/null) && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "matrix --features exits 0" || fail "matrix --features exit ${ec} (expected 0)"
    # header row contains 'Feature' and each configuration id as a column
    local hdr; hdr=$(printf '%s' "$out" | grep -F "| Feature" | head -1 || true)
    [ -n "$hdr" ] && pass "header row has 'Feature' column" || fail "no 'Feature' header row"
    for c in CONF-PL-PETROL-001 CONF-PL-EV-001 CONF-PL-LUX-001; do
        printf '%s' "$hdr" | grep -qF "$c" && pass "header has column $c" || fail "header missing column $c"
    done
    # a selected feature/config cell shows ✓ — Electric is selected by the EV config
    local row; row=$(printf '%s' "$out" | grep -F "Features::Engine::Electric" | head -1 || true)
    [ -n "$row" ] && pass "row for Electric present" || fail "no row for Electric feature"
    printf '%s' "$row" | grep -qF "✓" && pass "Electric row shows ✓ for a selecting config" || fail "Electric row has no ✓"

    SCENARIO_NAME="default matrix still shows Requirement × Configuration view"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local d dec
    d=$("$SYSCRIBE" -m "$B" matrix 2>/dev/null) && dec=0 || dec=$?
    [ "${dec:-0}" -eq 0 ] && pass "matrix exits 0" || fail "matrix exit ${dec} (expected 0)"
    printf '%s' "$d" | grep -qF "| Requirement |" && pass "default matrix has Requirement column" || fail "default matrix lost Requirement view"
    printf '%s' "$d" | grep -qF "REQ-PL-EV-001" && pass "default matrix lists a requirement" || fail "default matrix missing requirement row"
}
