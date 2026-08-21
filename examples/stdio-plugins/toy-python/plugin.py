#!/usr/bin/env python3
"""A minimal stdio-subprocess plugin (ADR-SYS-PLUGIN-002).

Demonstrates the whole protocol in stdlib-only Python: no build step, no
dependencies, deliberately different from a compile-to-WASM toolchain to show
"any language, zero toolchain" directly.

Reads one JSON request object from stdin, parses every `*.toy` file under the
package directory the request names, and writes one JSON envelope to stdout.
Logging (if any) MUST go to stderr — stdout is reserved for exactly one
envelope object.

The toy DSL recognised here is intentionally trivial — one declaration per
line:

    part <Name>: <PartDef|RequirementDef|...>[, id=<STABLE-ID>][, satisfies=<REQ-ID>][, doc="..."]

`satisfies=<REQ-ID>` demonstrates a link *from* the foreign model *to* the
native Syscribe model: it becomes an ordinary `satisfies:` list on the
emitted element's frontmatter, resolved by Syscribe's normal resolver after
the merge — the plugin doesn't need to know whether the target actually
exists; that's Syscribe's job during `validate`.

The point is demonstrating the mechanism, not building a real parser.
"""

import json
import re
import sys
from pathlib import Path

LINE_RE = re.compile(
    r"^part\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?P<type>[A-Za-z]+)"
    r"(?:\s*,\s*id=(?P<id>[^,]+))?"
    r"(?:\s*,\s*satisfies=(?P<satisfies>[^,]+))?"
    r"(?:\s*,\s*doc=\"(?P<doc>[^\"]*)\")?\s*$"
)


def parse_toy_file(path: Path):
    elements = []
    diagnostics = []
    for lineno, raw_line in enumerate(path.read_text().splitlines(), start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        m = LINE_RE.match(line)
        if not m:
            diagnostics.append(
                {
                    "severity": "warning",
                    "message": f"{path.name}:{lineno}: unrecognised line, skipped",
                }
            )
            continue
        elem = {"qname": m.group("name"), "type": m.group("type")}
        if m.group("id"):
            elem["id"] = m.group("id").strip()
        if m.group("satisfies"):
            elem["satisfies"] = [m.group("satisfies").strip()]
        if m.group("doc"):
            elem["doc"] = m.group("doc")
        elements.append(elem)
    return elements, diagnostics


def main() -> int:
    request = json.load(sys.stdin)
    package_dir = Path(request["packageDir"])

    elements = []
    diagnostics = []
    for toy_file in sorted(package_dir.rglob("*.toy")):
        file_elements, file_diagnostics = parse_toy_file(toy_file)
        elements.extend(file_elements)
        diagnostics.extend(file_diagnostics)

    json.dump({"elements": elements, "diagnostics": diagnostics}, sys.stdout)
    return 0


if __name__ == "__main__":
    sys.exit(main())
