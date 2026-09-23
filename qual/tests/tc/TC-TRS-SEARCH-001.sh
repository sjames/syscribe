tc_TRS_SEARCH_001() {
    local F="$1"; local M="$F/TC-TRS-SEARCH-001/model" out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _jq() { printf '%s' "$3" | jq -e "$2" >/dev/null 2>&1 && pass "$1" || fail "$1 — got: $(printf '%s' "$3" | head -c 600)"; }
    local init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"qual","version":"1"}}}'

    # Fixture: "watchdog" occurs 4× in the short body+name of PartDef Arch::Supervisor,
    # 3× in REQ-S1-001 (slightly longer), 1× in the long body of REQ-S1-002, nowhere else.
    # "satcom" occurs only in REQ-S1-012, which appliesWhen Features::Link::Sat
    # (deselected by CONF-S1-001). "radio" occurs in REQ-S1-010/011 (draft: 011) and 012.

    _scn "The most relevant element ranks first"
    local wd; wd=$("$SYSCRIBE" -m "$M" search-text watchdog --json 2>/dev/null) || true
    _jq "exactly the 3 watchdog-bearing elements match" '.total==3 and (.results|length)==3' "$wd"
    _jq "densest element (Arch::Supervisor) ranks first" '.results[0].qname=="Arch::Supervisor"' "$wd"
    _jq "ranking is Supervisor > REQ-S1-001 > REQ-S1-002 (single mention last)" '[.results[].qname]==["Arch::Supervisor","Power::REQ-S1-001","Power::REQ-S1-002"]' "$wd"
    _jq "scores strictly descending" '[.results[].score] as $s | all(range(1;$s|length); $s[.-1] > $s[.])' "$wd"
    out=$("$SYSCRIBE" -m "$M" search-text watchdog 2>/dev/null) || true
    local order; order=$(printf '%s\n' "$out" | grep -oE '^(Arch::Supervisor|REQ-S1-00[0-9])' | paste -sd, || true)
    [ "$order" = "Arch::Supervisor,REQ-S1-001,REQ-S1-002" ] && pass "text output lists hits best-first" || fail "text order: '$order'"
    out=$("$SYSCRIBE" -m "$M" search-text watchdog --limit 1 --json 2>/dev/null) || true
    _jq "--limit 1 returns only the top hit, total still 3" '(.results|length)==1 and .results[0].qname=="Arch::Supervisor" and .total==3' "$out"

    _scn "Each result carries a marked snippet"
    _jq "each result carries id, qname, type, score, snippet" 'all(.results[]; (keys)==["id","qname","score","snippet","type"])' "$wd"
    _jq "Requirement results carry their stable id; PartDef has null id + qname" '.results[0].id==null and .results[1].id=="REQ-S1-001" and .results[2].id=="REQ-S1-002"' "$wd"
    _jq "types reported (PartDef, Requirement, Requirement)" '[.results[].type]==["PartDef","Requirement","Requirement"]' "$wd"
    _jq "every snippet marks the term as **watchdog**" 'all(.results[]; .snippet|ascii_downcase|contains("**watchdog**"))' "$wd"
    _jq "long body snippet is a window around the hit (REQ-S1-002)" '.results[2].snippet|startswith("…") and contains("**watchdog**")' "$wd"

    _scn "--type restricts the searched set"
    out=$("$SYSCRIBE" -m "$M" search-text watchdog --type Requirement --json 2>/dev/null) || true
    _jq "every result has type Requirement" '(.results|length)>0 and all(.results[]; .type=="Requirement")' "$out"
    _jq "PartDef dropped: exactly REQ-S1-001, REQ-S1-002 in order" '.total==2 and [.results[].id]==["REQ-S1-001","REQ-S1-002"]' "$out"
    out=$("$SYSCRIBE" -m "$M" search-text radio --status draft --json 2>/dev/null) || true
    _jq "--status draft: only the draft radio requirements (011, 012)" '([.results[].id]|sort)==["REQ-S1-011","REQ-S1-012"]' "$out"

    _scn "An empty query is a usage error"
    "$SYSCRIBE" -m "$M" search-text "" >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "search-text \"\" exits 1" || fail "search-text \"\" exit $rc (expected 1)"
    "$SYSCRIBE" -m "$M" search-text "   " >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "whitespace-only query exits 1" || fail "whitespace-only query exit $rc (expected 1)"

    _scn "--config searches only the variant"
    out=$("$SYSCRIBE" -m "$M" search-text satcom --json 2>/dev/null) || true
    _jq "unprojected: satcom finds REQ-S1-012" '.total==1 and .results[0].id=="REQ-S1-012"' "$out"
    out=$("$SYSCRIBE" -m "$M" search-text satcom --config CONF-S1-001 --json 2>/dev/null) || true
    _jq "--config CONF-S1-001: gated REQ-S1-012 absent (0 results)" '.total==0 and (.results|length)==0' "$out"
    out=$("$SYSCRIBE" -m "$M" search-text radio --type Requirement --config CONF-S1-001 --json 2>/dev/null) || true
    _jq "--config: radio (Requirements) finds 010, 011 but not 012" '([.results[].id]|sort)==["REQ-S1-010","REQ-S1-011"]' "$out"
    "$SYSCRIBE" -m "$M" search-text radio --config bogus >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "--config bogus exits 1" || fail "--config bogus exit $rc (expected 1)"

    _scn "The MCP search_text tool matches the CLI"
    out=$(printf '%s\n' "$init" \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
        '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search_text","arguments":{"query":"watchdog"}}}' \
        | timeout 20 "$SYSCRIBE" -m "$M" mcp 2>/dev/null) || true
    printf '%s' "$out" | jq -se '[.[]|select(.id==2)|.result.tools[]|select(.name=="search_text")][0].annotations.readOnlyHint==true' >/dev/null 2>&1 \
        && pass "search_text tool advertised with readOnlyHint true" || fail "search_text tool not advertised read-only"
    local tool cli
    tool=$(printf '%s' "$out" | jq -S 'select(.id==3)|.result.content[0].text|fromjson' 2>/dev/null) || true
    cli=$(printf '%s' "$wd" | jq -S . 2>/dev/null) || true
    [ -n "$tool" ] && [ "$tool" = "$cli" ] && pass "MCP search_text document (incl. order) == search-text --json" || fail "MCP search_text differs: $(printf '%s' "$tool" | head -c 400)"
}
