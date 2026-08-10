tc_TRS_VAL_017() {
    local F="$1"; local M="$F/TC-TRS-VAL-017"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    local out; out=$("$SYSCRIBE" -m "$M" validate 2>&1 || true)

    _scn "a Part usage typed by a documented PartDef raises no W600"
    printf '%s' "$out" | grep 'x\.md' | grep -q 'W600' \
        && fail "W600 unexpectedly raised for x.md (typed by documented DocumentedDef)" \
        || pass "no W600 for x.md"

    _scn "the documented PartDef itself raises no W600"
    printf '%s' "$out" | grep 'DocumentedDef\.md' | grep -q 'W600' \
        && fail "W600 unexpectedly raised for DocumentedDef.md" \
        || pass "no W600 for DocumentedDef.md"

    _scn "an undocumented PartDef still raises W600"
    printf '%s' "$out" | grep 'UndocumentedDef\.md' | grep -q 'W600' \
        && pass "W600 raised for UndocumentedDef.md" \
        || fail "W600 not raised for UndocumentedDef.md"

    _scn "a Part typed by an equally-undocumented PartDef still raises W600"
    printf '%s' "$out" | grep 'y\.md' | grep -q 'W600' \
        && pass "W600 raised for y.md" \
        || fail "W600 not raised for y.md"

    _scn "a Part with an unresolvable typedBy: still raises W600"
    printf '%s' "$out" | grep 'z\.md' | grep -q 'W600' \
        && pass "W600 raised for z.md" \
        || fail "W600 not raised for z.md"
}
