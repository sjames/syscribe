tc_TRS_CLI_009() {
    local F="$1"; local M="$F/TC-TRS-CFLD-002/model"; local G="$F/TC-TRS-PROJ-007/gate"
    local o e rc c want

    # _c9_bad <model> <expected-stderr-substring> <args...>: expect a usage error.
    _c9_bad() {
        local m="$1" want="$2"; shift 2
        o=$("$SYSCRIBE" -m "$m" "$@" 2>/dev/null) && rc=0 || rc=$?
        e=$("$SYSCRIBE" -m "$m" "$@" 2>&1 >/dev/null || true)
        if [ "$rc" -ne 0 ] && [ -z "$o" ] && grep -qF -- "$want" <<<"$e"; then
            pass "$* → exit $rc, stderr names '$want'"
        else
            fail "$* → exit $rc, stdout ${#o} bytes, stderr: $(head -c 200 <<<"$e")"
        fi
    }
    # _c9_ok <model> <args...>: expect exit 0.
    _c9_ok() {
        local m="$1"; shift
        "$SYSCRIBE" -m "$m" "$@" >/dev/null 2>&1 && rc=0 || rc=$?
        [ "$rc" -eq 0 ] && pass "$* → exit 0" || fail "$* → exit $rc (expected 0)"
    }

    SCENARIO_NAME="invalid enumerated values are rejected and the valid values listed"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    _c9_bad "$M" "downstream, upstream, both" impact Engine --direction sideways
    _c9_bad "$M" "text, json, dot"           impact Engine --format xml
    _c9_bad "$M" "text, html, json"          n2 --format xml
    _c9_bad "$M" "text, json"                behavioral-coverage --format xml
    _c9_bad "$M" "cyclonedx, spdx"           sbom --format xml
    _c9_bad "$G" "cmake, c-header, makefile" build-config --config CONF-PROJ7-WDT-001 --format xml
    _c9_bad "$G" "--all-configs"             build-config --all-configs --format cmake
    _c9_bad "$M" "cargo-json, junit"         validate --results "$M/Engine.md" --format bogus
    _c9_bad "$M" "nonexist.json"             validate --results "$M/nonexist.json"
    _c9_bad "$M" "bogus"                     diagram render Engine --view bogus
    [ "$rc" -eq 2 ] && pass "diagram keeps clap's usage exit code 2" || fail "diagram --view bogus exit $rc (expected clap's 2)"

    SCENARIO_NAME="non-integer counts are rejected"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    while IFS= read -r c; do
        want=$(awk '{for(i=1;i<=NF;i++) if ($i ~ /^--/) f=$i} END{print f}' <<<"$c")
        # shellcheck disable=SC2086
        _c9_bad "$M" "$want" $c
    done <<'CMDS'
n2 --depth abc
impact Engine --depth -1
behavioral-coverage --depth x
summarize --depth x
stats --package-top-n x
digest --limit x
digest --offset x
search-text engine --limit x
topics --top x
clusters --k x
verification-depth --min-levels x
CMDS
    _c9_bad "$M" "--min-width" diagram render Engine --min-width wide

    SCENARIO_NAME="unknown options are rejected on the checked commands"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    while IFS= read -r c; do
        # shellcheck disable=SC2086
        _c9_bad "$M" "--bogus" $c
        [ "$rc" -eq 1 ] || fail "$c → exit $rc (expected usage exit 1)"
    done <<'CMDS'
validate --bogus
list PartDef --bogus
show Engine --bogus
trace Engine --bogus
why Engine --bogus
who-verifies Engine --bogus
impact Engine --bogus
links Engine --bogus
refs Engine --bogus
export --bogus
find engine --bogus
ls --bogus
tree --bogus
extref X --bogus
n2 --bogus
behavioral-coverage --bogus
sbom --bogus
build-config --bogus
stats --bogus
digest --bogus
search-text engine --bogus
summarize --bogus
topics --bogus
clusters --bogus
verification-depth --bogus
CMDS
    _c9_bad "$M" "--depth" impact Engine --depth
    _c9_bad "$M" "--format=json" n2 --format=json
    o=$("$SYSCRIBE" -m /nonexistent/model/dir list PartDef --bogus 2>&1 >/dev/null || true)
    grep -qF -- "--bogus" <<<"$o" && pass "option check runs before model resolution" \
        || fail "no model dir: expected the --bogus usage error, got: $(head -c 160 <<<"$o")"

    SCENARIO_NAME="valid options still work"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    _c9_ok "$M" impact Engine --direction both --format json --depth 2
    _c9_ok "$M" impact Engine --direction upstream --format dot --kinds satisfies
    _c9_ok "$M" n2 --format html --depth 1 --interfaces-only --allocations
    _c9_ok "$M" behavioral-coverage --format json --depth 2 --uncovered-only --include-planned
    _c9_ok "$M" sbom --format spdx --include-tests
    _c9_ok "$G" build-config --config CONF-PROJ7-WDT-001 --format cmake --prefix MY_ --no-validate
    _c9_ok "$G" build-config --all-configs --format json
    _c9_ok "$M" validate --json --deny=W005 --max-warnings=100 --file Engine.md
    _c9_ok "$M" list PartDef --where=custom.supplier=Bosch --json --tag x --status draft
    _c9_ok "$M" show Engine --no-related
    _c9_ok "$M" export --ndjson
    _c9_ok "$M" find engine --where custom.supplier
    _c9_ok "$M" ls --where custom.supplier
    _c9_ok "$M" diagram render Engine --view REQ
}
