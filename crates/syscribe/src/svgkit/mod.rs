//! Shared SVG drawing helpers: system-font text metrics and the per-element-type
//! colour theme. Kept from the retired CLI diagram toolkit (`ADR-SYS-VIS-001`,
//! `REQ-TRS-VIS-013`) because the MagicGrid report (`mgreport`, `REQ-TRS-MG-016`)
//! draws its grid with them.

pub mod metrics;
pub mod theme;
pub mod types;
