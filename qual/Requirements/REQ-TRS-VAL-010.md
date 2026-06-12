---
id: REQ-TRS-VAL-010
type: Requirement
name: Tool shall verify testFunctions resolve in sourceFile across languages (W009)
status: draft
reqDomain: software
verificationMethod: test
---

For every `TestCase` whose `sourceFile:` exists on disk, the tool **shall** verify that each `testFunctions[].function` resolves to a test/function definition in that file, and **shall** emit warning `W009` for any entry that does not resolve. This provides *function-level* model↔code traceability beyond `W004`'s *file-level* existence check, catching tests that were renamed or deleted while the source file survived.

The tool **shall** provide built-in, language-aware matchers for:

| Language | Extensions | Recognises |
|---|---|---|
| Rust | `.rs` | `fn name` (incl. `#[test]`, `#[kani::proof]`) |
| Java | `.java` | method declarations |
| C / C++ | `.c .h .cpp .cc .cxx .hpp .hh .hxx .ino` | function definitions, GoogleTest `TEST`/`TEST_F`/`TEST_P` |
| Kotlin | `.kt .kts` | `fun name`, backtick test names |
| Shell | `.sh .bash .bats .zsh .ksh` | POSIX `name()`, `function name`, bats `@test "..."` |

For any other file type that represents a test (e.g. `.robot`, `.feature`, `.txt`, generated manifests), the tool **shall** apply a generic whole-token fallback so the test name is still confirmed present. Per-extension matcher patterns **shall** be overridable via a `[matchers]` table in `<model_root>/.syscribe.toml`.

The function name is the last segment of `testFunctions[].function` after splitting on `::`, `.`, `#`, or `/`.

**Source:** Issue #1 (function-level traceability); §11.12 (`W009`)

**Acceptance criteria:** A `TestCase` with a `sourceFile` containing the named test in any supported language (or a generic file) produces no `W009`; a renamed/deleted function (or a generic file lacking the named test) produces `W009`.
