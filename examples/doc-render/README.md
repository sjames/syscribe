# Values of the model in a LaTeX document

A report written in LaTeX states its design values by reference, and `render` writes the file
with each reference replaced by the current value and unit (GH #530). Nothing is copied by
hand, so `refs` and `lint-docs` see every use.

- `model/` — a heater on a test rig: a feature parameter `Limits.maxTemp` (120 degC by default,
  130 in `CONF-HTR-001`), a derived parameter `Heater.onTimeMs`, and an attribute `Rig.settle`.
- `model/.syscribe.toml` — the `[documents]` table: where the documents are, and that `.tex`
  files write a reference as `<<Elem.attr>>` (braces clash with LaTeX).
- `doc/chapter.tex` — the document.
- `model/Views/` — two table views: the feature parameters and the requirements. A view with
  `columns:` declares the table a document prints, and `view render` writes it (GH #531).

```bash
# The document with its values, written for LaTeX: 120\,\textdegree{}C, 240\,ms, 150\,ms
syscribe -m examples/doc-render/model/ render examples/doc-render/doc/chapter.tex

# The same document for one variant: 130\,\textdegree{}C, 260\,ms
syscribe -m examples/doc-render/model/ render examples/doc-render/doc/chapter.tex --config CONF-HTR-001 --out build/chapter.tex

# Stale ids, broken item references and broken value references in the document (W098, W099)
syscribe -m examples/doc-render/model/ lint-docs examples/doc-render/doc/

# Which document sections a value touches
syscribe -m examples/doc-render/model/ refs Rig.settle
syscribe -m examples/doc-render/model/ refs Limits.maxTemp
```

Tables come from the views, with no generator code:

```bash
# A longtable of the parameters, with the values of one variant
syscribe -m examples/doc-render/model/ view render Views::Parameters --format latex --config CONF-HTR-001 --out build/parameters.tex

# The same table as Markdown or CSV
syscribe -m examples/doc-render/model/ view render Views::Parameters --format markdown
syscribe -m examples/doc-render/model/ view render Views::RequirementList --format csv
```

A reference that does not resolve stops the render with `<file>:<line>: E331: …`, and nothing
is written. See `syscribe help render` and `docs/model-guide/value-references.md`.
