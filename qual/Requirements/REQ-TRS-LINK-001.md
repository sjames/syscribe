---
id: REQ-TRS-LINK-001
type: Requirement
name: "Tool shall resolve a hosted source URL for each element from a [links] config"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** resolve, for each **file-backed** model element, a **hosted source URL**
from a `[links]` table in `<model_root>/.syscribe.toml`, so generated diagrams and reports can
link back to where the model is hosted (GitHub/GitLab/a static site). This is the foundation
consumed by the per-surface requirements ([[REQ-TRS-LINK-002]]–[[REQ-TRS-LINK-005]]).

### Configuration

```toml
[links]
base_url = "https://github.com/<org>/<repo>/blob/main/model"   # the 90% case
# optional escape hatch for hosted HTML sites / custom refs / anchors:
url_template = "https://github.com/<org>/<repo>/blob/{ref}/model/{path}"
ref = "main"
```

- `base_url` (string) and/or `url_template` (string) — at least one enables the feature.
- `ref` (string, optional) — substituted for `{ref}` in the template.
- Unknown keys are ignored (parses alongside `[profiles]`/`[matchers]`/`[remote]`).

### URL resolution

For a file-backed element whose file path **relative to the model root** is `<path>` (always
forward-slashed):

- If `url_template` is set, substitute placeholders: `{path}`, `{qname}` (qualified name),
  `{id}` (stable id, else empty), `{ref}`.
- Else if `base_url` is set, the URL is `base_url` (one trailing `/` trimmed) + `/` + `<path>`.
- Path **segments** are URL-encoded (e.g. a space → `%20`); the `/` separators are preserved.
- A **package** resolves to its `_index.md` path. An element **not backed by its own file**
  (e.g. an attribute that is a YAML key in a parent) resolves to **no** URL.

### Opt-in and advisory

- When `[links]` declares neither `base_url` nor `url_template`, **no** element resolves to a
  URL and the feature is inert (diagrams/reports are exactly as before).
- URL resolution is **advisory**: it never affects validation findings or exit codes, and never
  rewrites the model.

**Source:** user feature request — clickable links from diagram/report elements to the hosted
model. Foundation for [[REQ-TRS-LINK-002]] (SVG), [[REQ-TRS-LINK-003]] (Mermaid),
[[REQ-TRS-LINK-004]] (Markdown), [[REQ-TRS-LINK-005]] (live server).

**Acceptance criteria:**

- With `base_url = "https://h/x/blob/main/model"`, an element at `UAV/Avionics/FlightController.md`
  resolves to `https://h/x/blob/main/model/UAV/Avionics/FlightController.md`.
- With a `url_template`, `{path}`/`{qname}`/`{id}`/`{ref}` are substituted; a file path with a
  space is percent-encoded in `{path}` while `/` separators are preserved.
- A package element resolves to its `_index.md`; a YAML-key attribute resolves to no URL.
- With no `[links]` table (or an empty one), no element resolves to a URL.
