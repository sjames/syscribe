---
type: Requirement
id: REQ-TRS-BUILD-010
name: "build-config subcommand with --config and --format options"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BUILD-000]
breakdownAdr: Decisions::BuildExportADR
tags:
  - cli
  - build-integration
---

Syscribe shall provide a `build-config` subcommand with the following interface:

```
syscribe -m <root> build-config --config <id> --format <format> [--prefix <string>] [--no-validate]
```

The subcommand shall resolve the named `Configuration`, compute the full build variable
set (see `REQ-TRS-BUILD-020`), and write the result to standard output in the requested
format. The exit code shall be `0` on success and non-zero on any error.

### Supported formats

| `--format` value | Output style |
|---|---|
| `cmake` | `set(VAR value)` statements, suitable for `include()` in a `CMakeLists.txt` |
| `c-header` | `#define VAR value` lines wrapped in a header guard |
| `makefile` | `VAR := value` assignments, suitable for `include` in a `Makefile` |
| `env` | `export VAR=value` lines, suitable for shell `source` |
| `json` | A single JSON object mapping variable names to their resolved values |
| `kconfig` | `CONFIG_VAR=y` / `CONFIG_VAR=value` lines (Zephyr/Linux Kconfig convention) |

Each format shall include a header comment identifying the generating command and the
configuration ID, so the output is traceable back to the model.

### `--prefix` option

When `--prefix <string>` is supplied, every emitted variable name shall be prefixed
with `<string>`. This allows multiple feature-model outputs to coexist in one build
namespace (e.g. `--prefix UAV_` yields `UAV_ENABLE_ABS`).

### `--no-validate` flag

By default the subcommand runs SAT validation on the configuration before generating
output (see `REQ-TRS-BUILD-017`). `--no-validate` suppresses this check and generates
output unconditionally.
