---
type: FeatureDef
id: FEAT-FX-030
name: Ecu
groupKind: mandatory
---

Engine control unit. Uses the **legacy** `groupKind: mandatory` membership style
(no `mandatory:` field). As a mandatory child it is forced whenever the parent
Power feature is present, so it must be treated as a core/forced feature.
