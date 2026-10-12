---
id: REQ-TRS-VIS-032
type: Requirement
name: Diagram exports are the same bytes on every run and machine, and a model's committed renders can be checked in CI
status: verified
reqDomain: software
verificationMethod: test
---

The diagram sizer **shall** measure text with advance widths compiled into the tool, never with
the fonts installed on the machine, so the same model **shall** export to byte-identical
PlantUML, Mermaid, SVG and DOT on every run and every machine. `diagram export --all` **shall**
render every `Diagram` element of the model, in qualified-name order, to
`<out-dir>/<qname path>.<ext>` (default `<model>/generated`, SVG unless `--format` says
otherwise), rewrite only files whose content changed and remove files of that format which no
diagram renders any more. `--check` **shall** write nothing and exit 1 naming every file that
differs, is missing or is left over. Every SVG shape and edge **shall** carry `sysml:ref`.

**Source:** GH #339.

**Acceptance criteria:** (a) two exports of model_gti in every format are identical; (b) the
committed model_gti renders pass `--check`; (c) editing a drawn element makes `--check` fail
until regenerated; (d) a missing and a left-over render are each reported; (e) every SVG shape
and edge names its element; (f) the sizer's widths are the bundled table's whatever fonts are
installed.
