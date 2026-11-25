---
type: Document
name: HeaterReport
template: model:../doc/report.tex.tmpl
rendered: model:../doc/build/report.tex
artefacts: [model:../doc/build/report.pdf]
version: "0.2"
audience: customer
---

The design report of the heater: its limits, its parameters and its requirements.
`syscribe doc build HeaterReport` renders `doc/report.tex.tmpl` to `doc/build/report.tex`;
LaTeX makes the PDF from that.
