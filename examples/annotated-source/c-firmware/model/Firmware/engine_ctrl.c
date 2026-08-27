// Ordinary C firmware — Syscribe's native Markdown+YAML parser never reads
// this file directly. It exists only because `Firmware/_index.md` declares
// `annotationFormat: c-linecomment`, which hands the whole package subtree
// to the in-process comment-marker scanner instead.

#include <stdint.h>

// @syscribe
// type: Part
// name: EngineController
// satisfies: [REQ-ENGCTL-100]
// doc: >-
//   Holds and reports the commanded target engine RPM. `doc:` is the one
//   marker key that isn't itself a frontmatter field — it becomes this
//   element's documentation body, since a marker has no separate region
//   below a `---` the way a hand-authored `.md` file does.
//
// The marker block above is literal Syscribe frontmatter YAML — the exact
// same grammar a hand-authored `.md` file's frontmatter uses. No
// `implementedBy:` is set here on purpose: the scanner auto-fills it with
// this marker's own source location (`Firmware/engine_ctrl.c`), which is
// what `W563` on `Firmware/_index.md` confirms after a scan.

static uint32_t target_rpm;

void engine_ctrl_set_target_rpm(uint32_t rpm) {
    target_rpm = rpm;
}

uint32_t engine_ctrl_get_target_rpm(void) {
    return target_rpm;
}
