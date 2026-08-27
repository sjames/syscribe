---
type: Package
name: Firmware
annotationFormat: c-linecomment
marker: '//\s*@syscribe\b'
include: ["**/*.c", "**/*.h"]
exclude: ["**/vendor/**"]
---

Ordinary C firmware, not a foreign notation — this package's `.c`/`.h` files
are scanned in-process for `// @syscribe` comment markers instead of being
handed to an external plugin. See `../engine_ctrl.c` for the marker itself.
