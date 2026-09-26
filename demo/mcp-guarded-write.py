#!/usr/bin/env python3
"""Replay the README's guarded-write demo against a throwaway copy of model_auto/.

    python3 demo/mcp-guarded-write.py [path/to/syscribe]

Drives a real `syscribe mcp` server over stdio (stdlib only, no MCP SDK) and plays
the part of an agent: propose a requirement with a dangling reference, watch the
dry-run report and the commit gate refuse it, fix it, commit. Nothing in your
working tree is touched. Record it with asciinema (see demo/README.md).
"""
import json, os, pathlib, shutil, subprocess, sys, tempfile, time

ROOT = pathlib.Path(__file__).resolve().parent.parent
BIN = sys.argv[1] if len(sys.argv) > 1 else str(ROOT / "target" / "debug" / "syscribe")
PACE = float(os.environ.get("DEMO_PACE", "0"))  # seconds between lines; ~0.8 when recording

tmp = pathlib.Path(tempfile.mkdtemp(prefix="syscribe-demo-"))
model = tmp / "model_auto"
shutil.copytree(ROOT / "model_auto", model)
# The copy lives outside the repo, so the repo-relative PlantUML style path in the
# demo model's config would not resolve; drop it to keep the demo output focused.
cfg = model / ".syscribe.toml"
cfg.write_text("".join(l for l in cfg.read_text().splitlines(True) if "style_file" not in l))

p = subprocess.Popen([BIN, "-m", str(model), "mcp"], stdin=subprocess.PIPE,
                     stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
_id = 0
def rpc(method, params=None, notify=False):
    global _id
    msg = {"jsonrpc": "2.0", "method": method, **({"params": params} if params else {})}
    if not notify:
        _id += 1; msg["id"] = _id
    p.stdin.write(json.dumps(msg) + "\n"); p.stdin.flush()
    while not notify:
        r = json.loads(p.stdout.readline())
        if r.get("id") == _id:
            return r
def tool(name, args):
    return json.loads(rpc("tools/call", {"name": name, "arguments": args})["result"]["content"][0]["text"])
def say(s=""):
    print(s, flush=True); time.sleep(PACE)

rpc("initialize", {"protocolVersion": "2024-11-05", "capabilities": {},
                   "clientInfo": {"name": "demo-agent", "version": "0"}})
rpc("notifications/initialized", notify=True)

def proposal(parent):
    return {"qname": "Requirements::Safety::FaultLogging", "type": "Requirement",
            "fields": {"name": "Monitor shall log every detected fault", "status": "draft",
                       "reqDomain": "software", "asilLevel": "B", "verificationMethod": "test",
                       "derivedFrom": [parent], "breakdownAdr": "ADR-ENG-SAFE-001"},
            "doc": "The safety monitor **shall** log every detected fault with a timestamp."}

def report(r):
    d = r["validationDelta"]
    for e in d["newErrors"]:
        say(f"  ✗ new error   {e['code']}  {e['message']}")
    for w in d["newWarnings"]:
        say(f"  ! new warning {w['code']}  {w['message']}")
    if not (d["newErrors"] or d["newWarnings"]):
        say("  ✓ no new errors or warnings")
    if r.get("reason"):
        say(f"  ⛔ {r['reason']}")
    say(f"  written: {str(r['written']).lower()}")

say("$ claude mcp add syscribe -- syscribe -m model_auto/ mcp")
say()
say("agent> Add a requirement: the monitor must log every fault.")
say()
for parent, commit in (("REQ-ENG-SAFE-099", False), ("REQ-ENG-SAFE-099", True), ("REQ-ENG-SAFE-000", False), ("REQ-ENG-SAFE-000", True)):
    dry = not commit
    say(f"agent> create_element  Requirements::Safety::FaultLogging  derivedFrom: [{parent}]  dry_run: {str(dry).lower()}")
    report(tool("create_element", {**proposal(parent), "dry_run": dry}))
    say()
say("result> wrote model_auto/Requirements/Safety/FaultLogging.md")
p.stdin.close(); p.wait(timeout=5)
shutil.rmtree(tmp, ignore_errors=True)
