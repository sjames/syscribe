tc_TRS_SYSMLV2_017() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-017/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "a package-relative typedBy: reference to a def in another package suppresses W007 on that def"
    local out; out=$("$SYSCRIBE" -m "$M" validate 2>&1 || true)
    # Services::Documented is referenced only via the package-relative
    # "typedBy: Services::Documented" that ingest.rs produces for
    # `part x : Services::Documented;` written inside `package System`.
    # Before REQ-TRS-SYSMLV2-017 this counted as unused.
    printf '%s' "$out" | grep -q "W007.*'SysML2::Services::Documented'" \
        && fail "W007 unexpectedly fired for Services::Documented (cross-package usage not tracked)" \
        || pass "no W007 for Services::Documented -- its cross-package usage is tracked"

    _scn "the same cross-package reference is a real, connectivity-visible TypedBy edge"
    local conn_out; conn_out=$("$SYSCRIBE" -m "$M" connectivity SysML2::System::Top::x 2>&1)
    printf '%s' "$conn_out" | grep -q '\[typedBy\] SysML2::Services::Documented' \
        && pass "connectivity shows a typedBy edge from Top::x to Services::Documented" \
        || fail "expected a typedBy edge to Services::Documented, got: $conn_out"

    _scn "a genuinely unused PartDef in the same model still raises W007"
    printf '%s' "$out" | grep -q "W007.*'SysML2::Services::Orphan'" \
        && pass "W007 still fires for the genuinely unused Orphan" \
        || fail "expected W007 for Services::Orphan (genuinely unused), got: $out"
}
