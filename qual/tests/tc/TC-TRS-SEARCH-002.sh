tc_TRS_SEARCH_002() {
    # Shares the TC-TRS-SEARCH-001 fixture: Requirements in Power (watchdog/battery),
    # Comms (radio/link; REQ-S1-012 "satellite" gated by Features::Link::Sat) and
    # Brake (hydraulic caliper); a PartDef in Arch; FeatureDefs in Features and
    # Features::Link.
    local F="$1"; local M="$F/TC-TRS-SEARCH-001/model" out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _jq() { printf '%s' "$3" | jq -e "$2" >/dev/null 2>&1 && pass "$1" || fail "$1 — got: $(printf '%s' "$3" | head -c 600)"; }
    local init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"qual","version":"1"}}}'

    _scn "Each package gets a distinctive keyword list"
    local full; full=$("$SYSCRIBE" -m "$M" topics --json 2>/dev/null) || true
    _jq "shape { packages: { <pkg>: [ {term, score} … ] } }" '(keys)==["packages"] and all(.packages[][]; (keys)==["score","term"] and (.term|type)=="string" and (.score|type)=="number")' "$full"
    _jq "one entry per Requirement package: Brake, Comms, Power (not Arch/Features)" '(.packages|keys)==["Brake","Comms","Power"]' "$full"
    _jq "each package's terms ordered by non-increasing score" 'all(.packages[]; [.[].score] as $s | all(range(1;$s|length); $s[.-1] >= $s[.]))' "$full"
    _jq "default --top is 10 terms per package" 'all(.packages[]; length==10)' "$full"
    _jq "distinctive top term per package: Power=watchdog, Comms=radio" '.packages.Power[0].term=="watchdog" and .packages.Comms[0].term=="radio"' "$full"
    _jq "Brake's top terms are its own vocabulary (caliper/hydraulic/pressure)" '[.packages.Brake[0:3][].term]|sort==["caliper","hydraulic","pressure"]' "$full"
    _jq "no package's list leaks another package's signature term" '(.packages.Power|map(.term)|index("radio")==null) and (.packages.Comms|map(.term)|index("watchdog")==null)' "$full"
    local sw
    for sw in the shall and of to a when over; do
        _jq "no stopword '$sw' in any package" "[.packages[][].term]|index(\"$sw\")==null" "$full"
    done

    _scn "--top bounds the term count"
    out=$("$SYSCRIBE" -m "$M" topics --top 3 --json 2>/dev/null) || true
    _jq "each package lists exactly 3 terms" 'all(.packages[]; length==3)' "$out"
    local want; want=$(printf '%s' "$full" | jq -c '.packages|map_values(.[0:3])' 2>/dev/null) || true
    _jq "--top 3 lists are the prefixes of the default lists" ".packages==$want" "$out"

    _scn "--type selects the element type and spans its packages"
    out=$("$SYSCRIBE" -m "$M" topics --type FeatureDef --json 2>/dev/null) || true
    _jq "packages = every package containing a FeatureDef (Features, Features::Link)" '(.packages|keys)==["Features","Features::Link"]' "$out"
    _jq "Features::Link's top term is lora (from its FeatureDefs, not Requirements)" '.packages["Features::Link"][0].term=="lora"' "$out"
    out=$("$SYSCRIBE" -m "$M" topics --type PartDef --json 2>/dev/null) || true
    _jq "--type PartDef: only Arch" '(.packages|keys)==["Arch"] and .packages.Arch[0].term=="watchdog"' "$out"

    _scn "--config projects before computing"
    "$SYSCRIBE" -m "$M" topics --config bogus >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "--config bogus exits 1" || fail "--config bogus exit $rc (expected 1)"
    _jq "unprojected Comms terms include satellite (from REQ-S1-012)" '.packages.Comms|map(.term)|index("satellite")!=null' "$full"
    out=$("$SYSCRIBE" -m "$M" topics --config CONF-S1-001 --json 2>/dev/null) || true
    _jq "--config CONF-S1-001: satellite/satcom gone from Comms" '.packages.Comms|map(.term)|(index("satellite")==null and index("satcom")==null)' "$out"

    _scn "The MCP topics tool matches the CLI"
    out=$(printf '%s\n' "$init" \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
        '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"topics","arguments":{}}}' \
        | timeout 20 "$SYSCRIBE" -m "$M" mcp 2>/dev/null) || true
    printf '%s' "$out" | jq -se '[.[]|select(.id==2)|.result.tools[]|select(.name=="topics")][0].annotations.readOnlyHint==true' >/dev/null 2>&1 \
        && pass "topics tool advertised with readOnlyHint true" || fail "topics tool not advertised read-only"
    local tool cli
    tool=$(printf '%s' "$out" | jq -S 'select(.id==3)|.result.content[0].text|fromjson' 2>/dev/null) || true
    cli=$(printf '%s' "$full" | jq -S . 2>/dev/null) || true
    [ -n "$tool" ] && [ "$tool" = "$cli" ] && pass "MCP topics document == topics --json" || fail "MCP topics differs: $(printf '%s' "$tool" | head -c 400)"
}
