---
type: Package
name: Vendor
repoImports:
  - repo: cell
    qname: Configurations
    as: CellConfs
  - repo: cell
    qname: Features
    as: CellFeatures
---
Mounts the cell tier's configurations at `Vendor::CellConfs` and its feature model at
`Vendor::CellFeatures`.
