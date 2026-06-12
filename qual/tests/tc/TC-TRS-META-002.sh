tc_TRS_META_002() {
    local F="$1"; local M="$F/TC-TRS-META-002/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "a stereotyped element renders a «Critical» banner plus the type-keyword banner"
    local pump; pump=$("$SYSCRIBE" -m "$M" diagram render System::Pump 2>/dev/null || true)
    { printf '%s' "$pump" | grep -qF "<svg" \
        && printf '%s' "$pump" | grep -qF "«Critical»" \
        && printf '%s' "$pump" | grep -qF "«part def»"; } \
        && pass "Pump shows «Critical» and «part def»" \
        || fail "Pump missing the stereotype banner or the type-keyword banner"

    _scn "the «Critical» banner uses the shared stereotype styling (stereotype_fg, italic)"
    printf '%s' "$pump" | grep -F "«Critical»" | grep -qF 'font-style="italic"' \
        && pass "«Critical» banner is italic (stereotype styling)" \
        || fail "«Critical» banner does not use the stereotype styling"

    _scn "an element with no application renders no spurious stereotype banner"
    local plain; plain=$("$SYSCRIBE" -m "$M" diagram render System::Plain 2>/dev/null || true)
    { printf '%s' "$plain" | grep -qF "«part def»" \
        && ! printf '%s' "$plain" | grep -qF "«Critical»"; } \
        && pass "Plain shows only its type-keyword banner" \
        || fail "Plain has a spurious stereotype banner"
}
