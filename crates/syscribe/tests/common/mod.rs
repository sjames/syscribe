//! Shared harness for the `syscribe mcp` and `syscribe lsp` integration tests.
//!
//! Spawns the subcommand as a subprocess and drives it over its stdio transport:
//! MCP uses newline-delimited JSON-RPC 2.0 (`Mcp`), LSP uses `Content-Length`-framed
//! JSON-RPC 2.0 (`Lsp`). These tests are written before the implementation exists —
//! until the subcommand is implemented they fail (the process errors out and stdout
//! closes, or the framing never matches), which is the intended red state.
#![allow(dead_code)]

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Recursively copy a directory tree.
fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

/// Copy the shared fixture model into a unique temp directory so write-tool tests
/// can mutate it without disturbing the checked-in fixture. Returns the model root.
pub fn fixture_copy() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let base = std::env::temp_dir().join(format!("syscribe-mcp-test-{}-{}-{}", std::process::id(), nanos, n));
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model");
    let dst = base.join("model");
    copy_dir(&src, &dst);
    dst
}

/// A minimal MCP client over the subprocess's stdio.
pub struct Mcp {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    next_id: i64,
    pub model_root: PathBuf,
    /// Server-initiated notifications observed while awaiting responses.
    pub notifications: Vec<Value>,
}

impl Mcp {
    /// Spawn `syscribe mcp -m <model_root>` and return the client.
    pub fn start(model_root: &Path) -> Mcp {
        Mcp::start_with_args(model_root, &[])
    }

    /// Spawn `syscribe mcp <extra…> -m <model_root>` and return the client.
    pub fn start_with_args(model_root: &Path, extra: &[&str]) -> Mcp {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_syscribe"));
        cmd.arg("mcp");
        for a in extra {
            cmd.arg(a);
        }
        let mut child = cmd
            .arg("-m")
            .arg(model_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn syscribe mcp");
        let stdin = child.stdin.take().unwrap();
        let reader = BufReader::new(child.stdout.take().unwrap());
        Mcp { child, stdin, reader, next_id: 1, model_root: model_root.to_path_buf(), notifications: Vec::new() }
    }

    fn send(&mut self, msg: &Value) {
        let line = serde_json::to_string(msg).unwrap();
        self.stdin.write_all(line.as_bytes()).unwrap();
        self.stdin.write_all(b"\n").unwrap();
        self.stdin.flush().unwrap();
    }

    /// Read JSON-RPC messages until one carries the given id; returns its `result`
    /// (or panics on error / EOF — EOF is the red-state failure).
    fn read_result(&mut self, id: i64) -> Value {
        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line).expect("read line");
            if n == 0 {
                panic!("EOF from server before response to id {id} (server not implemented yet?)");
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let v: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue, // ignore non-JSON log noise
            };
            if v.get("id").and_then(|x| x.as_i64()) == Some(id) {
                if let Some(err) = v.get("error") {
                    panic!("JSON-RPC error for id {id}: {err}");
                }
                return v.get("result").cloned().unwrap_or(Value::Null);
            }
            // A server-initiated notification (method, no id): record and keep reading.
            if v.get("id").is_none() && v.get("method").is_some() {
                self.notifications.push(v);
            }
        }
    }

    /// True if a notification with the given method has been observed.
    pub fn saw_notification(&self, method: &str) -> bool {
        self.notifications.iter().any(|n| n.get("method").and_then(|m| m.as_str()) == Some(method))
    }

    pub fn logging_set_level(&mut self, level: &str) -> Value {
        self.request("logging/setLevel", json!({"level": level}))
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        self.read_result(id)
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(&json!({"jsonrpc": "2.0", "method": method, "params": params}));
    }

    /// Perform the initialize handshake; returns the initialize result.
    pub fn initialize(&mut self) -> Value {
        let res = self.request(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "syscribe-test", "version": "0"}
            }),
        );
        self.notify("notifications/initialized", json!({}));
        res
    }

    pub fn tools_list(&mut self) -> Value {
        self.request("tools/list", json!({}))
    }

    pub fn resources_list(&mut self) -> Value {
        self.request("resources/list", json!({}))
    }

    pub fn resources_read(&mut self, uri: &str) -> Value {
        self.request("resources/read", json!({"uri": uri}))
    }

    pub fn prompts_list(&mut self) -> Value {
        self.request("prompts/list", json!({}))
    }

    pub fn prompts_get(&mut self, name: &str) -> Value {
        self.request("prompts/get", json!({"name": name, "arguments": {}}))
    }

    pub fn resource_templates_list(&mut self) -> Value {
        self.request("resources/templates/list", json!({}))
    }

    /// Request argument completion for a resource-template argument.
    pub fn complete_resource(&mut self, uri: &str, arg_name: &str, value: &str) -> Value {
        self.request(
            "completion/complete",
            json!({
                "ref": {"type": "ref/resource", "uri": uri},
                "argument": {"name": arg_name, "value": value}
            }),
        )
    }

    /// Call a tool and return the raw `tools/call` result (including `isError` and the
    /// `content` array). Used by error-path tests that must inspect the error flag rather
    /// than the parsed payload.
    pub fn call_tool_raw(&mut self, name: &str, args: Value) -> Value {
        self.request("tools/call", json!({"name": name, "arguments": args}))
    }

    /// True while the spawned server process is still running (no exit status yet).
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Call a tool and parse the structured JSON payload out of the first text content.
    pub fn call_tool(&mut self, name: &str, args: Value) -> Value {
        let res = self.request("tools/call", json!({"name": name, "arguments": args}));
        // MCP wraps tool output in result.content[].text; our tools return JSON text.
        let text = res
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or_else(|| panic!("tool {name} returned no text content: {res}"));
        serde_json::from_str(text).unwrap_or_else(|_| json!({"_raw": text}))
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Hash of a directory tree's file contents + relative paths, for "disk unchanged" checks.
pub fn dir_hash(root: &Path) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    fn walk(dir: &Path, base: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let p = entry.path();
            let ft = entry.file_type().unwrap();
            if ft.is_symlink() {
                // Skip symlinks: a "tree unchanged" hash compares real model files only,
                // and following a dir symlink here would error.
                continue;
            }
            if ft.is_dir() {
                walk(&p, base, out);
            } else {
                let rel = p.strip_prefix(base).unwrap().to_string_lossy().to_string();
                out.push((rel, std::fs::read(&p).unwrap()));
            }
        }
    }
    walk(root, root, &mut entries);
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut h = std::collections::hash_map::DefaultHasher::new();
    entries.hash(&mut h);
    h.finish()
}

/// Convert a filesystem path to a `file://` URI (test-only; no percent-encoding —
/// fixture paths under the OS temp dir don't contain characters that need it).
pub fn file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

/// A minimal LSP client over a subprocess's stdio, using `Content-Length`-framed
/// JSON-RPC 2.0 (the standard LSP transport), for driving `syscribe lsp`.
pub struct Lsp {
    child: Child,
    /// `None` once `shutdown()` has closed it — tower-lsp's read loop only stops on
    /// stdin EOF (processing the `exit` notification alone does not end the process),
    /// so the client must close its write end to let the server terminate.
    stdin: Option<ChildStdin>,
    reader: BufReader<std::process::ChildStdout>,
    next_id: i64,
    pub model_root: PathBuf,
    /// Server-initiated notifications observed while awaiting a response (includes
    /// `textDocument/publishDiagnostics`).
    pub notifications: Vec<Value>,
}

impl Lsp {
    /// Spawn `syscribe lsp -m <model_root>` and return the client.
    pub fn start(model_root: &Path) -> Lsp {
        let child = Command::new(env!("CARGO_BIN_EXE_syscribe"))
            .arg("lsp")
            .arg("-m")
            .arg(model_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn syscribe lsp");
        let mut child = child;
        let stdin = child.stdin.take().unwrap();
        let reader = BufReader::new(child.stdout.take().unwrap());
        Lsp { child, stdin: Some(stdin), reader, next_id: 1, model_root: model_root.to_path_buf(), notifications: Vec::new() }
    }

    fn send(&mut self, msg: &Value) {
        let body = serde_json::to_vec(msg).unwrap();
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        let stdin = self.stdin.as_mut().expect("stdin not yet closed");
        stdin.write_all(header.as_bytes()).unwrap();
        stdin.write_all(&body).unwrap();
        stdin.flush().unwrap();
    }

    /// Read one `Content-Length`-framed JSON-RPC message.
    fn read_message(&mut self) -> Value {
        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line).expect("read lsp header line");
            if n == 0 {
                panic!("EOF from lsp server while reading headers (server not implemented yet?)");
            }
            let line = line.trim_end_matches(['\r', '\n']);
            if line.is_empty() {
                break;
            }
            if let Some(v) = line.strip_prefix("Content-Length:") {
                content_length = v.trim().parse().ok();
            }
        }
        let len = content_length.expect("Content-Length header present in lsp response");
        let mut buf = vec![0u8; len];
        self.reader.read_exact(&mut buf).expect("read lsp message body");
        serde_json::from_slice(&buf).expect("parse lsp message body as JSON")
    }

    /// Read messages until one carries the given id; returns its `result` (panics on
    /// a JSON-RPC error or EOF). Notifications seen along the way are recorded.
    fn read_result(&mut self, id: i64) -> Value {
        loop {
            let v = self.read_message();
            if v.get("id").and_then(|x| x.as_i64()) == Some(id) {
                if let Some(err) = v.get("error") {
                    panic!("JSON-RPC error for id {id}: {err}");
                }
                return v.get("result").cloned().unwrap_or(Value::Null);
            }
            if v.get("id").is_none() && v.get("method").is_some() {
                self.notifications.push(v);
            }
        }
    }

    /// Read (and consume) the next unconsumed notification with the given method —
    /// first from already-buffered notifications (FIFO), else by reading new messages.
    /// Returns its `params`. Panics on EOF. Consuming lets a test wait for a *second*
    /// occurrence of the same method (e.g. two publishDiagnostics rounds) without the
    /// first match short-circuiting every later call.
    pub fn wait_for_notification(&mut self, method: &str) -> Value {
        if let Some(pos) = self.notifications.iter().position(|n| n.get("method").and_then(|m| m.as_str()) == Some(method)) {
            return self.notifications.remove(pos).get("params").cloned().unwrap_or(Value::Null);
        }
        loop {
            let v = self.read_message();
            if v.get("id").is_none() && v.get("method").and_then(|m| m.as_str()) == Some(method) {
                return v.get("params").cloned().unwrap_or(Value::Null);
            }
            if v.get("id").is_none() && v.get("method").is_some() {
                self.notifications.push(v);
            }
        }
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        let mut msg = json!({"jsonrpc": "2.0", "id": id, "method": method});
        // Omit `params` entirely for parameterless methods (e.g. `shutdown`) — a
        // strict server (tower-lsp) rejects an explicit `"params": null`.
        if !params.is_null() {
            msg["params"] = params;
        }
        self.send(&msg);
        self.read_result(id)
    }

    fn notify(&mut self, method: &str, params: Value) {
        let mut msg = json!({"jsonrpc": "2.0", "method": method});
        if !params.is_null() {
            msg["params"] = params;
        }
        self.send(&msg);
    }

    /// Perform the `initialize`/`initialized` handshake; returns the `initialize` result.
    pub fn initialize(&mut self) -> Value {
        let root_uri = file_uri(&self.model_root);
        let res = self.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": root_uri,
                "capabilities": {},
                "clientInfo": {"name": "syscribe-test", "version": "0"}
            }),
        );
        self.notify("initialized", json!({}));
        res
    }

    pub fn did_open(&mut self, path: &Path, language_id: &str, text: &str) {
        self.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": file_uri(path),
                    "languageId": language_id,
                    "version": 1,
                    "text": text,
                }
            }),
        );
    }

    pub fn did_change(&mut self, path: &Path, version: i64, text: &str) {
        self.notify(
            "textDocument/didChange",
            json!({
                "textDocument": {"uri": file_uri(path), "version": version},
                "contentChanges": [{"text": text}]
            }),
        );
    }

    pub fn did_save(&mut self, path: &Path, text: Option<&str>) {
        let mut params = json!({"textDocument": {"uri": file_uri(path)}});
        if let Some(text) = text {
            params["text"] = json!(text);
        }
        self.notify("textDocument/didSave", params);
    }

    pub fn did_change_watched_files(&mut self, path: &Path, change_type: u32) {
        self.notify(
            "workspace/didChangeWatchedFiles",
            json!({"changes": [{"uri": file_uri(path), "type": change_type}]}),
        );
    }

    pub fn hover(&mut self, path: &Path, line: u32, character: u32) -> Value {
        self.request(
            "textDocument/hover",
            json!({"textDocument": {"uri": file_uri(path)}, "position": {"line": line, "character": character}}),
        )
    }

    pub fn definition(&mut self, path: &Path, line: u32, character: u32) -> Value {
        self.request(
            "textDocument/definition",
            json!({"textDocument": {"uri": file_uri(path)}, "position": {"line": line, "character": character}}),
        )
    }

    pub fn references(&mut self, path: &Path, line: u32, character: u32) -> Value {
        self.request(
            "textDocument/references",
            json!({
                "textDocument": {"uri": file_uri(path)},
                "position": {"line": line, "character": character},
                "context": {"includeDeclaration": true}
            }),
        )
    }

    pub fn workspace_symbol(&mut self, query: &str) -> Value {
        self.request("workspace/symbol", json!({"query": query}))
    }

    /// True while the spawned server process is still running (no exit status yet).
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Send `shutdown` followed by `exit`, close stdin (tower-lsp's read loop only
    /// stops on EOF — the `exit` notification alone does not end the process), and
    /// wait for the process to terminate, returning its exit code.
    pub fn shutdown(&mut self) -> Option<i32> {
        self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        self.stdin.take(); // drop closes the pipe, sending EOF
        self.child.wait().ok().and_then(|s| s.code())
    }
}

impl Drop for Lsp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
