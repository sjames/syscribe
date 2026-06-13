---
id: REQ-TRS-TYPE-017
type: Requirement
name: "Tool shall support a native Asset element type for ISO/SAE 21434 §15.3 asset identification"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** support a native element type **`Asset`** for formal asset identification as required by ISO/SAE 21434 §15.3.

## Asset element schema

- **`id`** (required): stable identifier matching pattern `^ASSET(-[A-Z0-9]{2,12})+-[0-9]{3,8}$` (e.g. `ASSET-001`, `ASSET-IPC-001`).
- **`name`** (required): human-readable asset name.
- **`status`** (required): lifecycle status string.
- **`cybersecurityProperties`** (optional): list of strings from `confidentiality | integrity | availability | authenticity`.
- **`assetOwner`** (optional): qualified name or id of the architecture element that owns this asset.
- **`relatedSafetyGoal`** (optional): qualified name or id of a `SafetyGoal` for co-engineering link.

## DamageScenario.assets extension

`DamageScenario` **shall** accept an optional **`assets`** field (string or list) referencing `Asset` elements.

## Validation rules

- Missing `id` on Asset → **E861**.
- Missing `name` on Asset → **E861**.
- Missing `status` on Asset → **E861**.
- Asset `id` not matching the ASSET-* pattern → **E862**.
- Invalid `cybersecurityProperties` value → **E863**.
- `DamageScenario.assets` ref not resolving or resolving to non-Asset → **E864**.
- Warning **W810**: an Asset with no `DamageScenario.assets` or `ThreatScenario` reference (unreferenced asset).

**Acceptance criteria:**

- An Asset element with valid `id`, `name`, `status` validates cleanly.
- Invalid `id` pattern triggers E862.
- Invalid `cybersecurityProperties` entry triggers E863.
- `DamageScenario.assets: ASSET-001` resolves correctly; pointing to a non-Asset triggers E864.
- `list Asset` displays the asset list.
