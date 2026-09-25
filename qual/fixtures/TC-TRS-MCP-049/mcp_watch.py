"""Minimal MCP client for TC-TRS-MCP-049.

Usage: mcp_watch.py <syscribe> <model-root> <file> <old-name> <new-name> [server args...]

Starts `syscribe mcp [server args...] -m <model-root>`, performs the initialize
handshake, reads REQ-WATCH-001's name with `get_element`, rewrites <file> on disk
(replacing <old-name> with <new-name>, atomically via rename) and then polls
`get_element` — up to 20 s for a watching server; for the full 3 s window under
`--no-watch`, where nothing may change. Finally closes stdin and waits for the
process to exit. Prints one JSON object:

  {"before": str, "after": str, "watchReloads": int, "listChanged": bool, "exited": bool}
"""
import json
import os
import queue
import subprocess
import sys
import threading
import time


def main():
    exe, root, path, old, new = sys.argv[1:6]
    server_args = sys.argv[6:]
    no_watch = "--no-watch" in server_args
    proc = subprocess.Popen([exe, "mcp", *server_args, "-m", root], stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    lines = queue.Queue()

    def pump():
        for raw in proc.stdout:
            lines.put(raw)
        lines.put(None)

    threading.Thread(target=pump, daemon=True).start()
    notes = []
    next_id = [1]

    def send(msg):
        proc.stdin.write((json.dumps(msg) + "\n").encode())
        proc.stdin.flush()

    def request(method, params):
        mid = next_id[0]
        next_id[0] += 1
        send({"jsonrpc": "2.0", "id": mid, "method": method, "params": params})
        while True:
            raw = lines.get(timeout=30)
            if raw is None:
                raise SystemExit("server closed stdout")
            try:
                msg = json.loads(raw)
            except ValueError:
                continue
            if msg.get("id") == mid:
                if "error" in msg:
                    raise SystemExit("error: %s" % msg["error"])
                return msg.get("result")
            if "method" in msg and "id" not in msg:
                notes.append(msg)

    def name():
        res = request("tools/call", {"name": "get_element", "arguments": {"ref": "REQ-WATCH-001"}})
        return json.loads(res["content"][0]["text"]).get("name")

    out = {}
    try:
        request("initialize", {"protocolVersion": "2025-06-18", "capabilities": {},
                               "clientInfo": {"name": "qual", "version": "0"}})
        send({"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})
        out["before"] = name()

        with open(path) as f:
            text = f.read()
        tmp = path + ".tmp"
        with open(tmp, "w") as f:
            f.write(text.replace(old, new))
        os.replace(tmp, path)

        start = time.time()
        after = out["before"]
        while time.time() - start < (3 if no_watch else 20):
            after = name()
            if after == new and not no_watch:
                break
            time.sleep(0.1)
        out["after"] = name()
        out["watchReloads"] = sum(
            1 for n in notes
            if n.get("method") == "notifications/message"
            and n.get("params", {}).get("data", {}).get("event") == "reload"
            and n.get("params", {}).get("data", {}).get("source") == "watch")
        out["listChanged"] = any(n.get("method") == "notifications/resources/list_changed" for n in notes)
    finally:
        proc.stdin.close()
        try:
            proc.wait(timeout=20)
            out["exited"] = True
        except subprocess.TimeoutExpired:
            out["exited"] = False
            proc.kill()
    print(json.dumps(out))


if __name__ == "__main__":
    main()
