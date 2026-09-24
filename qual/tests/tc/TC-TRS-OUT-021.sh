tc_TRS_OUT_021() {
    local F="$1"; local M="$F/TC-TRS-OUT-021/model" out full rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # _jq "<description>" '<jq boolean filter>' "<json>"
    _jq() { printf '%s' "$3" | jq -e "$2" >/dev/null 2>&1 && pass "$1" || fail "$1 — got: $(printf '%s' "$3" | head -c 600)"; }
    local init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"qual","version":"1"}}}'

    # Fixture: 9 native requirements. Power (5, incl. Power::Cells) + Comms (4).
    # status: approved 4 (001, 002, 010, Q21P), draft 4 (003, 011, 012, Q21C), deprecated 1 (013).
    # REQ-Q21-001 silLevel 2, REQ-Q21-002 asilLevel B, the other 7 declare neither.
    # Verified (active TC): REQ-Q21-001, REQ-Q21-010 — both approved.
    # REQ-Q21P-001 is a parent of REQ-Q21C-001; REQ-Q21-012 appliesWhen Sat (off in CONF-Q21-001).

    _scn "The digest reports the total and every facet"
    full=$("$SYSCRIBE" -m "$M" stats --json 2>/dev/null) || true
    _jq "output is valid JSON object" 'type=="object"' "$full"
    _jq "total = 9" '.total==9' "$full"
    _jq "carries total, facets, coverage, orphans" 'has("total") and has("facets") and has("coverage") and has("orphans")' "$full"
    _jq "facets = {status,reqDomain,silLevel,asilLevel,package,tags}" '(.facets|keys)==(["asilLevel","package","reqDomain","silLevel","status","tags"])' "$full"
    _jq "status histogram exact" '.facets.status=={"approved":4,"draft":4,"deprecated":1}' "$full"
    _jq "reqDomain histogram exact" '.facets.reqDomain=={"software":6,"hardware":2,"system":1}' "$full"
    _jq "package histogram exact (top-level)" '.facets.package=={"Power":5,"Comms":4}' "$full"
    _jq "tags histogram exact" '.facets.tags=={"power":2,"safety":1,"comms":2,"security":1}' "$full"
    _jq "silLevel: 1×SIL2 + QM/none 7 (neither declared)" '.facets.silLevel=={"2":1,"QM/none":7}' "$full"
    _jq "asilLevel: 1×B + QM/none 7 (neither declared)" '.facets.asilLevel=={"B":1,"QM/none":7}' "$full"

    _scn "Coverage equals the coverage/matrix computation"
    out=$(printf '%s\n' "$init" \
        '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"coverage","arguments":{}}}' \
        | timeout 20 "$SYSCRIBE" -m "$M" mcp --read-only 2>/dev/null) || true
    local cov; cov=$(printf '%s' "$out" | jq -c 'select(.id==2)|.result.content[0].text|fromjson' 2>/dev/null) || true
    _jq "coverage tool verifiedCount = 2 (sanity)" '.verifiedCount==2' "$cov"
    local vc; vc=$(printf '%s' "$cov" | jq -r '.verifiedCount' 2>/dev/null) || true
    _jq "stats coverage.verified == coverage tool verifiedCount ($vc)" ".coverage.verified==${vc:-null}" "$full"
    local ul pm; ul=$(printf '%s' "$cov" | jq -r '.unverifiedLeaves|length' 2>/dev/null) || true
    pm=$(printf '%s' "$cov" | jq -r '.parentsMissingIntegrationTest|length' 2>/dev/null) || true
    _jq "stats unverifiedLeaves == coverage tool list length ($ul)" ".coverage.unverifiedLeaves==${ul:-null}" "$full"
    _jq "stats parentsMissingIntegration == coverage tool list length ($pm)" ".coverage.parentsMissingIntegration==${pm:-null}" "$full"

    _scn "A parent requirement is excluded from the orphan sets (GH #37)"
    _jq "parent REQ-Q21P-001 not in unsatisfiedRequirements" '.orphans.ids.unsatisfiedRequirements|index("REQ-Q21P-001")==null' "$full"
    _jq "parent REQ-Q21P-001 not in unverifiedRequirements" '.orphans.ids.unverifiedRequirements|index("REQ-Q21P-001")==null' "$full"
    _jq "untraced contains neither parent nor child" '.orphans.ids.untraced|(index("REQ-Q21P-001")==null and index("REQ-Q21C-001")==null)' "$full"
    _jq "untraced is exactly the 7 unlinked leaves" '.orphans.ids.untraced==["REQ-Q21-001","REQ-Q21-002","REQ-Q21-003","REQ-Q21-010","REQ-Q21-011","REQ-Q21-012","REQ-Q21-013"]' "$full"
    _jq "the (unverified) child is still reported unverified" '.orphans.ids.unverifiedRequirements|index("REQ-Q21C-001")!=null' "$full"

    _scn "--group-by re-keys a facet by top-level package"
    out=$("$SYSCRIBE" -m "$M" stats --group-by status --json 2>/dev/null) || true
    _jq "byPackage keyed by top-level package {Comms,Power}" '(.facets.byPackage|keys)==["Comms","Power"]' "$out"
    _jq "Power status histogram exact (incl. nested Cells)" '.facets.byPackage.Power=={"approved":3,"draft":2}' "$out"
    _jq "Comms status histogram exact" '.facets.byPackage.Comms=={"approved":1,"draft":2,"deprecated":1}' "$out"
    _jq "flat facets.status absent" '.facets|has("status")|not' "$out"

    _scn "An unknown --group-by facet is a usage error"
    local err
    err=$("$SYSCRIBE" -m "$M" stats --group-by bogus 2>&1 >/dev/null) && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "exit code 1" || fail "exit code $rc (expected 1)"
    for f in status reqDomain silLevel asilLevel tags; do
        printf '%s' "$err" | grep -q "expected one of:.*\b$f\b" && pass "stderr names valid facet $f" || fail "stderr does not name $f: $err"
    done

    _scn "Scoping filters restrict the counted set but not coverage"
    out=$("$SYSCRIBE" -m "$M" stats --status approved --json 2>/dev/null) || true
    _jq "--status approved: total = 4" '.total==4' "$out"
    _jq "--status approved: status facet only approved" '.facets.status=={"approved":4}' "$out"
    _jq "--status approved: coverage.verified unchanged (2)" '.coverage.verified==2' "$out"
    # draft set contains none of the verified requirements: a narrowed coverage would be 0.
    out=$("$SYSCRIBE" -m "$M" stats --status draft --json 2>/dev/null) || true
    _jq "--status draft: total = 4" '.total==4' "$out"
    _jq "--status draft: coverage.verified still 2 (whole model)" '.coverage.verified==2' "$out"
    out=$("$SYSCRIBE" -m "$M" stats --tag comms --json 2>/dev/null) || true
    _jq "--tag comms: total = 2, Comms only" '.total==2 and .facets.package=={"Comms":2}' "$out"

    _scn "--config projects the digest onto a variant"
    out=$("$SYSCRIBE" -m "$M" stats --config CONF-Q21-001 --json 2>/dev/null) || true
    _jq "total drops 9 → 8" '.total==8' "$out"
    _jq "draft 4 → 3, Comms 4 → 3, software 6 → 5, QM/none 7 → 6" '.facets.status.draft==3 and .facets.package.Comms==3 and .facets.reqDomain.software==5 and .facets.silLevel["QM/none"]==6' "$out"
    _jq "REQ-Q21-012 absent from all orphan id sets" '[.orphans.ids[][]]|index("REQ-Q21-012")==null' "$out"
    "$SYSCRIBE" -m "$M" stats --config bogus --json >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "--config bogus exits 1" || fail "--config bogus exit $rc (expected 1)"

    _scn "The MCP stats tool returns the same document as the CLI"
    out=$(printf '%s\n' "$init" \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
        '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"stats","arguments":{}}}' \
        | timeout 20 "$SYSCRIBE" -m "$M" mcp 2>/dev/null) || true
    printf '%s' "$out" | jq -se '[.[]|select(.id==2)|.result.tools[]|select(.name=="stats")][0].annotations.readOnlyHint==true' >/dev/null 2>&1 \
        && pass "stats tool advertised with readOnlyHint true" || fail "stats tool not advertised read-only"
    local tool cli
    tool=$(printf '%s' "$out" | jq -S 'select(.id==3)|.result.content[0].text|fromjson' 2>/dev/null) || true
    cli=$(printf '%s' "$full" | jq -S . 2>/dev/null) || true
    [ -n "$tool" ] && [ "$tool" = "$cli" ] && pass "MCP stats document == stats --json" || fail "MCP stats differs: $tool"
}
