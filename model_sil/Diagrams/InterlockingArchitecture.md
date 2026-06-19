---
type: Diagram
name: Interlocking Architecture
diagramKind: IBD
pumlMode: companion
pumlFile: ./InterlockingArchitecture.puml
subject: Deployment::Station1Interlocking
shapes:
  s-il:    {ref: "Deployment::Station1Interlocking",                                    kind: Part}
  s-vpa:   {ref: "Deployment::Station1Interlocking::vitalProcessorA",                   kind: Part, parent: s-il}
  s-vpb:   {ref: "Deployment::Station1Interlocking::vitalProcessorB",                   kind: Part, parent: s-il}
  s-sw:    {ref: "Deployment::Station1Interlocking::softwareImage",                     kind: Part, parent: s-il}
  s-rp:    {ref: "Deployment::Station1Interlocking::softwareImage::routeProcessor",     kind: Part, parent: s-sw}
  s-cc:    {ref: "Deployment::Station1Interlocking::softwareImage::conflictChecker",    kind: Part, parent: s-sw}
  s-pc:    {ref: "Deployment::Station1Interlocking::softwareImage::pointsController",   kind: Part, parent: s-sw}
  s-sc:    {ref: "Deployment::Station1Interlocking::softwareImage::signalController",   kind: Part, parent: s-sw}
  s-oc1:   {ref: "Deployment::Station1Interlocking::objectController1",                 kind: Part, parent: s-il}
  s-oc2:   {ref: "Deployment::Station1Interlocking::objectController2",                 kind: Part, parent: s-il}
  s-tc1:   {ref: "Deployment::Station1Interlocking::trackCircuit1",                     kind: Part, parent: s-il}
  s-tc2:   {ref: "Deployment::Station1Interlocking::trackCircuit2",                     kind: Part, parent: s-il}
  s-tc3:   {ref: "Deployment::Station1Interlocking::trackCircuit3",                     kind: Part, parent: s-il}
edges:
  e-vp-cross:  {source: s-vpa,  target: s-vpb,  kind: flow}
  e-vp-oc1:    {source: s-vpa,  target: s-oc1,  kind: flow}
  e-vp-oc2:    {source: s-vpa,  target: s-oc2,  kind: flow}
  e-vp-tc1:    {source: s-vpa,  target: s-tc1,  kind: flow}
  e-vp-tc2:    {source: s-vpa,  target: s-tc2,  kind: flow}
  e-vp-tc3:    {source: s-vpa,  target: s-tc3,  kind: flow}
  e-sw-vp:     {source: s-sw,   target: s-vpa,  kind: flow}
---

![Interlocking Architecture](./InterlockingArchitecture.svg)

Station 1 CBI architecture: the 2oo2D vital processor pair, Software Image
(Route Processor, Conflict Checker, Points Controller, Signal Controller),
Object Controllers for home signals and points/level crossing, and Track
Circuit sections. Field Bus connections run EN 50159 Cat 2.
