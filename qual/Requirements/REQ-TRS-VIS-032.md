---
id: REQ-TRS-VIS-032
type: Requirement
name: Diagram exports are the same bytes on every run and machine, and a model's committed renders can be checked in CI
status: verified
reqDomain: software
verificationMethod: test
---

The diagram sizer **shall** measure text with advance widths compiled into the tool, never with
the fonts installed on the machine. `diagram export --all` **shall** render every `Diagram`
element of the model, in qualified-name order, to `<out-dir>/<qname path>.<ext>` (default
`<model>/generated`, or `<model>/generated/<CONF-id>` with `--config`; SVG unless `--format`
says otherwise), with a PlantUML style `!include` relative to the file, so that its output is
byte-identical in every format on every run, machine and checkout location. It **shall** record
the files it wrote in `<out-dir>/.syscribe-generated`, rewrite only files whose content changed,
and remove only a plain file inside the directory that the record names, that no diagram renders
any more and that no `svgFile:` names; it **shall** refuse a directory that contains the model.
A diagram that fails to render (anything but a hand-authored body, a kind with no mapping in the
format or a companion picture) **shall** fail the run, keep its file and remove nothing.
`--check` **shall** write nothing and exit 1 naming every file that differs, is missing or is
left over, and every failure. Every SVG shape and edge **shall** carry `sysml:ref`.

**Source:** GH #339.

**Acceptance criteria:** (a) two exports of model_gti in every format are identical, and two
checkouts at different paths export the same SVG and PlantUML; (b) the committed model_gti
renders pass `--check`; (c) editing a drawn element makes `--check` fail until regenerated; (d) a
missing and a left-over render are each reported; (e) every SVG shape and edge names its
element; (f) the sizer's widths are the bundled table's whatever fonts are installed; (g) a file
the export did not write, a hand-drawn companion, a path through a link or out of the directory
is never removed; (h) a failed render fails the run and keeps its file; (i) one configuration
renders beside the whole model.
