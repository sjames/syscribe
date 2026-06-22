//! Language-aware function-definition matchers for function-level traceability (`W009`).
//!
//! A `TestCase` declares the source-code test functions it stands for via
//! `testFunctions[].function`, e.g. `mutex_tests::tests::acquire_returns_ok`.
//! `W004` only checks that `sourceFile:` exists; `W009` goes further and
//! verifies that each declared function actually *resolves* to a definition in
//! that file — catching tests that were renamed or deleted while the file
//! survived.
//!
//! Matching is purely lexical (regex over the file text), keyed by file
//! extension. Built-in matchers cover Rust, Java, C, C++ and Kotlin. Projects
//! can add or override per-extension patterns via a `[matchers]` table in
//! `<model_root>/.syscribe.toml` (see [`MatcherConfig::load_from_model_root`]).

use std::collections::HashMap;
use std::path::Path;

use regex::Regex;
use serde::Deserialize;

/// The function-name segment within `testFunctions[].function`, obtained by
/// splitting on the path separators used across languages and taking the last
/// non-empty segment. `a::b::c` → `c`, `Class.method` → `method`,
/// `classname#method` → `method`, `Suite.Name` → `Name`.
pub fn function_leaf(function_ref: &str) -> &str {
    function_ref
        .rsplit([':', '.', '#', '/'])
        .find(|s| !s.is_empty())
        .unwrap_or(function_ref)
}

/// Outcome of resolving one `testFunctions[].function` against its source file.
#[derive(Debug, PartialEq, Eq)]
pub enum FnResolution {
    /// A matching definition (or, for generic files, a matching name token) was found.
    Found,
    /// The file was searched but nothing matched the function name.
    NotFound,
    /// The source file could not be read.
    Unreadable,
}

/// Per-extension compiled matchers plus the function-name extractor.
#[derive(Debug, Clone)]
pub struct MatcherConfig {
    /// Lowercased extension (no dot) → ordered list of patterns. Capture group 1
    /// of each pattern is a function/test name defined in the file.
    by_ext: HashMap<String, Vec<Regex>>,
}

impl Default for MatcherConfig {
    fn default() -> Self {
        let mut by_ext: HashMap<String, Vec<Regex>> = HashMap::new();
        for (exts, patterns) in builtin_matchers() {
            let compiled: Vec<Regex> = patterns
                .iter()
                .map(|p| Regex::new(p).expect("built-in matcher pattern must compile"))
                .collect();
            for ext in exts {
                by_ext.insert((*ext).to_string(), compiled.clone());
            }
        }
        MatcherConfig { by_ext }
    }
}

impl MatcherConfig {
    /// Load matcher overrides from `<model_root>/.syscribe.toml` and merge them
    /// on top of the built-in defaults. A per-extension override **replaces**
    /// the built-in patterns for that extension. Returns the default config when
    /// the file is absent; on a malformed file, returns the default and the
    /// error string for the caller to surface.
    pub fn load_from_model_root(model_root: &Path) -> (Self, Option<String>) {
        let path = model_root.join(".syscribe.toml");
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => return (Self::default(), None), // absent → defaults
        };
        let raw: RawConfig = match toml::from_str(&text) {
            Ok(r) => r,
            Err(e) => {
                return (
                    Self::default(),
                    Some(format!(".syscribe.toml parse error: {}", e)),
                )
            }
        };
        let mut cfg = Self::default();
        let mut warn: Option<String> = None;
        for (ext, patterns) in raw.matchers {
            let mut compiled = Vec::with_capacity(patterns.len());
            for p in &patterns {
                match Regex::new(p) {
                    Ok(re) => compiled.push(re),
                    Err(e) => {
                        warn = Some(format!(
                            ".syscribe.toml: invalid regex for '.{}' ({}): {}",
                            ext, p, e
                        ));
                    }
                }
            }
            cfg.by_ext.insert(ext.to_ascii_lowercase(), compiled);
        }
        (cfg, warn)
    }

    /// Resolve one `function_ref` against `source_path`.
    ///
    /// When the file's extension has a language-specific matcher, the function
    /// name must match a captured definition. For any other file type (shell
    /// scripts are language-specific; arbitrary `.feature`, `.robot`, `.txt`,
    /// generated test manifests, etc. are not) a **generic fallback** searches
    /// for the name as a whole token — so any file that *represents* a test is
    /// supported, and a deleted/renamed test is still caught when its name token
    /// disappears.
    pub fn resolve(&self, source_path: &Path, function_ref: &str) -> FnResolution {
        let ext = source_path
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        let content = match std::fs::read_to_string(source_path) {
            Ok(c) => c,
            Err(_) => return FnResolution::Unreadable,
        };
        let leaf = function_leaf(function_ref);

        if let Some(patterns) = self.by_ext.get(&ext) {
            for re in patterns {
                for caps in re.captures_iter(&content) {
                    if let Some(name) = caps.get(1) {
                        if name.as_str() == leaf {
                            return FnResolution::Found;
                        }
                    }
                }
            }
            return FnResolution::NotFound;
        }

        // Generic fallback: whole-token search for the test name.
        if generic_token_present(&content, leaf) {
            FnResolution::Found
        } else {
            FnResolution::NotFound
        }
    }
}

/// True when `needle` appears in `haystack` bounded by non-identifier characters
/// (so `acquire_ok` does not match inside `acquire_ok_extended`). Used by the
/// generic fallback for file types without a language-specific matcher.
fn generic_token_present(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        let abs = start + pos;
        let before_ok = abs == 0
            || !haystack[..abs].chars().next_back().map(is_ident).unwrap_or(false);
        let after_idx = abs + needle.len();
        let after_ok = after_idx >= haystack.len()
            || !haystack[after_idx..].chars().next().map(is_ident).unwrap_or(false);
        if before_ok && after_ok {
            return true;
        }
        start = abs + needle.len();
    }
    false
}

/// TOML shape of `.syscribe.toml`. Only the `[matchers]` table is read today.
#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default)]
    matchers: HashMap<String, Vec<String>>,
}

/// Built-in function-definition patterns, grouped by the extensions they serve.
/// Capture group 1 must be the defined function/test name.
fn builtin_matchers() -> Vec<(&'static [&'static str], &'static [&'static str])> {
    vec![
        // Rust — `fn name`. Covers `#[test]` and `#[kani::proof]` (they precede `fn`).
        (
            &["rs"],
            &[r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)"],
        ),
        // Kotlin — `fun name(` and backtick test names `` fun `name with spaces`() ``.
        (
            &["kt", "kts"],
            &[
                r"\bfun\s+(?:<[^>]+>\s*)?(?:[A-Za-z_][\w.]*\.)?([A-Za-z_][\w]*)\s*\(",
                r"\bfun\s+`([^`]+)`",
            ],
        ),
        // Java — method declarations (modifiers + return type + name + params + body).
        (
            &["java"],
            &[r"\b[A-Za-z_$][\w$<>\[\].,\s]*\s+([A-Za-z_$][\w$]*)\s*\([^;{)]*\)\s*(?:throws[^{;]*)?\{"],
        ),
        // C / C++ — function definitions `name(...) {`, plus GoogleTest macros.
        (
            &["c", "h", "cpp", "cc", "cxx", "hpp", "hh", "hxx", "ino"],
            &[
                r"\b([A-Za-z_][\w]*)\s*\([^;{)]*\)\s*(?:const\s*)?\{",
                r"\bTEST(?:_F|_P)?\s*\(\s*\w+\s*,\s*([A-Za-z_]\w*)\s*\)",
            ],
        ),
        // Shell — POSIX `name() {`, `function name`, and bats `@test "desc" {`.
        // Covers shUnit2 (`test_foo`/`testFoo`), bats, and plain function tests.
        (
            &["sh", "bash", "bats", "zsh", "ksh"],
            &[
                r"(?m)^\s*(?:function\s+)?([A-Za-z_][\w]*)\s*\(\s*\)\s*\{",
                r"(?m)^\s*function\s+([A-Za-z_][\w]*)\b",
                r#"@test\s+"([^"]+)""#,
                r"@test\s+'([^']+)'",
            ],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(name: &str, content: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("syscribe-matchers-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        p
    }

    #[test]
    fn leaf_extraction() {
        assert_eq!(function_leaf("a::b::c"), "c");
        assert_eq!(function_leaf("Class.method"), "method");
        assert_eq!(function_leaf("com.x.Test#method"), "method");
        assert_eq!(function_leaf("Suite.Name"), "Name");
        assert_eq!(function_leaf("plain"), "plain");
    }

    #[test]
    fn rust_match() {
        let cfg = MatcherConfig::default();
        let p = write_tmp("a.rs", "#[test]\nfn acquire_returns_ok() { assert!(true); }\n");
        assert_eq!(cfg.resolve(&p, "m::tests::acquire_returns_ok"), FnResolution::Found);
        assert_eq!(cfg.resolve(&p, "m::tests::missing"), FnResolution::NotFound);
    }

    #[test]
    fn cpp_gtest_match() {
        let cfg = MatcherConfig::default();
        let p = write_tmp("a.cpp", "TEST_F(MutexTest, AcquireWhenFree) {\n  EXPECT_TRUE(true);\n}\n");
        assert_eq!(cfg.resolve(&p, "MutexTest.AcquireWhenFree"), FnResolution::Found);
    }

    #[test]
    fn c_function_match() {
        let cfg = MatcherConfig::default();
        let p = write_tmp("a.c", "void test_mutex_acquire(void) {\n  TEST_ASSERT(1);\n}\n");
        assert_eq!(cfg.resolve(&p, "test_mutex_acquire"), FnResolution::Found);
    }

    #[test]
    fn java_method_match() {
        let cfg = MatcherConfig::default();
        let p = write_tmp("a.java", "@Test\npublic void acquireReturnsOk() throws Exception {\n}\n");
        assert_eq!(cfg.resolve(&p, "com.x.MutexTest#acquireReturnsOk"), FnResolution::Found);
    }

    #[test]
    fn kotlin_backtick_match() {
        let cfg = MatcherConfig::default();
        let p = write_tmp("a.kt", "@Test\nfun `acquire returns ok when free`() {\n}\n");
        assert_eq!(cfg.resolve(&p, "acquire returns ok when free"), FnResolution::Found);
        let p2 = write_tmp("b.kt", "fun acquireReturnsOk() {}\n");
        assert_eq!(cfg.resolve(&p2, "MutexTest.acquireReturnsOk"), FnResolution::Found);
    }

    #[test]
    fn shell_function_and_bats() {
        let cfg = MatcherConfig::default();
        let p = write_tmp("a.sh", "test_mutex_acquire() {\n  assertTrue 0\n}\n");
        assert_eq!(cfg.resolve(&p, "test_mutex_acquire"), FnResolution::Found);
        assert_eq!(cfg.resolve(&p, "test_missing"), FnResolution::NotFound);

        let b = write_tmp("b.bats", "@test \"mutex acquire works\" {\n  run mutex\n}\n");
        assert_eq!(cfg.resolve(&b, "mutex acquire works"), FnResolution::Found);

        let f = write_tmp("c.bash", "function teardown {\n  :\n}\n");
        assert_eq!(cfg.resolve(&f, "teardown"), FnResolution::Found);
    }

    #[test]
    fn generic_fallback_for_unknown_extension() {
        let cfg = MatcherConfig::default();
        // Any file that represents a test: resolve by whole-token presence.
        let p = write_tmp("a.robot", "*** Test Cases ***\nAcquire When Free\n    Log    ok\n");
        assert_eq!(cfg.resolve(&p, "Acquire When Free"), FnResolution::Found);
        assert_eq!(cfg.resolve(&p, "Acquire When Held"), FnResolution::NotFound);

        // Whole-token boundary: substring of a larger identifier does not match.
        let q = write_tmp("b.txt", "test_acquire_extended\n");
        assert_eq!(cfg.resolve(&q, "test_acquire"), FnResolution::NotFound);
    }
}
