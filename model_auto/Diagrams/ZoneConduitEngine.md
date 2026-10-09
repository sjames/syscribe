---
type: Diagram
name: ZoneConduitEngine
diagramKind: ZoneConduit
subject: Security::Zones
---

IEC 62443 zones and conduits of the Engine ECU **derived from the model**: each zone is a compound node
holding its member parts and the security controls allocated to them, labelled with its target and
achieved security level (red with an `SL gap` badge when the achieved level falls short); each conduit
is an edge labelled `CD-… · SL achieved/required`, drawn as a heavy red dashed line when it is weaker
than the zones it joins. The levels and verdicts are those of `syscribe zones` and `syscribe conduits`.
