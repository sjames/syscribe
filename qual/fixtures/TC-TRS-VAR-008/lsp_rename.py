"""Minimal LSP client for TC-TRS-VAR-008.

Usage: lsp_rename.py <syscribe> <model-root> <file> <line> <character> <new-name>

Starts `syscribe lsp -m <model-root>`, performs the initialize handshake, sends one
`textDocument/rename` request, and prints the JSON-RPC response (the `result` or `error`
object, wrapped as {"result": ...} / {"error": ...}) as one line on stdout.
"""
import json
import pathlib
import subprocess
import sys


def send(proc, msg):
    body = json.dumps(msg).encode()
    proc.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
    proc.stdin.flush()


def read(proc):
    length = None
    while True:
        line = proc.stdout.readline()
        if not line:
            raise SystemExit("lsp closed stdout")
        line = line.strip()
        if not line:
            break
        if line.lower().startswith(b"content-length:"):
            length = int(line.split(b":", 1)[1])
    return json.loads(proc.stdout.read(length))


def response(proc, msg_id):
    while True:
        msg = read(proc)
        if msg.get("id") == msg_id:
            return msg


def main():
    exe, root, path, line, char, new_name = sys.argv[1:7]
    root_uri = pathlib.Path(root).resolve().as_uri()
    file_uri = pathlib.Path(path).resolve().as_uri()
    proc = subprocess.Popen([exe, "lsp", "-m", root], stdin=subprocess.PIPE,
                            stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    try:
        send(proc, {"jsonrpc": "2.0", "id": 1, "method": "initialize",
                    "params": {"processId": None, "rootUri": root_uri, "capabilities": {}}})
        response(proc, 1)
        send(proc, {"jsonrpc": "2.0", "method": "initialized", "params": {}})
        send(proc, {"jsonrpc": "2.0", "id": 2, "method": "textDocument/rename",
                    "params": {"textDocument": {"uri": file_uri},
                               "position": {"line": int(line), "character": int(char)},
                               "newName": new_name}})
        res = response(proc, 2)
        out = {"error": res["error"]} if "error" in res else {"result": res.get("result")}
        print(json.dumps(out))
        send(proc, {"jsonrpc": "2.0", "id": 3, "method": "shutdown"})
        response(proc, 3)
        send(proc, {"jsonrpc": "2.0", "method": "exit"})
    finally:
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()


if __name__ == "__main__":
    main()
