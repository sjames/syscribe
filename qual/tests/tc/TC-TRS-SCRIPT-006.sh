tc_TRS_SCRIPT_006() {
    local F="$1"
    local M="$F/TC-TRS-SCRIPT-006"
    local CLEAN="$F/TC-TRS-SCRIPT-006/clean"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "findings are namespaced and exit 1 on an error"
    local out rc=0
    out=$("$SYSCRIBE" -m "$M" scripts validate 2>&1) || rc=$?
    printf '%s' "$out" | grep -qF "supplier/NOSUP" \
        && pass "error finding rendered as <check>/<code>" || fail "namespaced error finding missing"
    printf '%s' "$out" | grep -qF "naming/DRAFT" \
        && pass "warning finding rendered as <check>/<code>" || fail "namespaced warning finding missing"
    printf '%s' "$out" | grep -qiF "checks.rhai" \
        && pass "source script shown" || fail "source script missing"
    [ "$rc" -eq 1 ] && pass "error-severity finding exits 1" || fail "exit $rc (expected 1)"

    _scn "clean run exits 0"
    rc=0; out=$("$SYSCRIBE" -m "$CLEAN" scripts validate 2>&1) || rc=$?
    [ "$rc" -eq 0 ] && pass "clean model exits 0" || fail "exit $rc (expected 0)"

    _scn "gate flags trip exit 2 on a warning-only model"
    # Build a model whose only check finding is a warning (no error), so the gate
    # (not an error finding) determines the exit code.
    local G; G=$(mktemp -d)
    cp -r "$M/." "$G/"
    rm -f "$G/Requirements/NoSupplier.md"   # remove the supplier-error source
    cat > "$G/.syscribe/scripts/checks.rhai" <<'RHAI'
fn naming(model) {
    for e in model.elements_of_type("Requirement") {
        if e.status == "draft" {
            finding(e, "DRAFT", "warning", "requirement is still draft");
        }
    }
}
register_check("naming", "Flag draft requirements", naming);
RHAI
    rc=0; out=$("$SYSCRIBE" -m "$G" scripts validate 2>&1) || rc=$?
    [ "$rc" -eq 0 ] && pass "warning-only run exits 0 ungated" || fail "warning-only exit $rc (expected 0)"
    rc=0; out=$("$SYSCRIBE" -m "$G" scripts validate --deny naming/DRAFT 2>&1) || rc=$?
    [ "$rc" -eq 2 ] && pass "--deny <check/code> trips exit 2" || fail "--deny exit $rc (expected 2)"
    rc=0; out=$("$SYSCRIBE" -m "$G" scripts validate --warnings-as-errors 2>&1) || rc=$?
    [ "$rc" -eq 2 ] && pass "--warnings-as-errors trips exit 2" || fail "warnings-as-errors exit $rc (expected 2)"
    rc=0; out=$("$SYSCRIBE" -m "$G" scripts validate --max-warnings 0 2>&1) || rc=$?
    [ "$rc" -eq 2 ] && pass "--max-warnings 0 trips exit 2" || fail "max-warnings exit $rc (expected 2)"
    rm -rf "$G"

    _scn "built-in validate is unaffected by check scripts"
    local with without
    with=$("$SYSCRIBE" -m "$M" validate 2>&1) || true
    local TMP; TMP=$(mktemp -d)
    cp -r "$M/." "$TMP/"
    rm -rf "$TMP/.syscribe/scripts"
    without=$("$SYSCRIBE" -m "$TMP" validate 2>&1) || true
    local wn on
    wn=$(printf '%s' "$with" | sed "s#$M#ROOT#g")
    on=$(printf '%s' "$without" | sed "s#$TMP#ROOT#g")
    [ "$wn" = "$on" ] && pass "built-in validate identical with/without scripts" \
        || fail "built-in validate output changed by scripts"
    printf '%s' "$with" | grep -qF "supplier/NOSUP" \
        && fail "built-in validate ran a check script" || pass "built-in validate did not run checks"
    rm -rf "$TMP"
}
