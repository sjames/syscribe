//! Shared SVG drawing helpers: system-font text metrics and the per-element-type
//! colour theme. Kept from the retired CLI diagram toolkit (`ADR-SYS-VIS-001`,
//! `REQ-TRS-VIS-013`) because the MagicGrid report (`mgreport`, `REQ-TRS-MG-016`)
//! draws its grid with them. The metrics now live in `syscribe-model`
//! (`vis::metrics`, `REQ-TRS-VIS-017`) so the diagram sizer and this report
//! share one implementation; the re-export keeps `mgreport` unchanged.

pub use syscribe_model::vis::metrics;
pub mod theme;
pub mod types;
