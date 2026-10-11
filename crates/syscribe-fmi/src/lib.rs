//! FMI Co-Simulation plant backend (REQ-TRS-FMI-001, GH #291). Implements the dependency-free
//! [`syscribe_model::plant`] seam with the `fmi` crates (FMI 2.0 and 3.0). Optional: built only with the
//! `fmi` cargo feature of `syscribe`.
//!
//! The model description is read here from `modelDescription.xml` rather than through `fmi-schema`: in
//! `fmi-schema` 0.8 the FMI 2.0 `ScalarVariable` `Real` drops its `unit` attribute, which real-world FMUs use
//! directly on variables, and units are what the dimension check needs.

mod md;
mod plant;

use std::path::Path;

use syscribe_model::plant::{FmuInfo, Plant, PlantBackend};

pub use md::{parse_model_description, ModelDescription};

/// The backend registered by `syscribe --features fmi`.
pub struct FmiBackend;

impl PlantBackend for FmiBackend {
    fn name(&self) -> String {
        format!("syscribe-fmi {} (fmi crate 0.8)", env!("CARGO_PKG_VERSION"))
    }

    fn inspect(&self, file: &Path) -> Result<FmuInfo, String> {
        md::inspect(file)
    }

    fn instantiate(&self, file: &Path, parameters: &[(String, f64)]) -> Result<Box<dyn Plant>, String> {
        plant::instantiate(file, parameters)
    }
}

/// Register the FMI backend with `syscribe-model`; false if a backend is already registered.
pub fn register() -> bool {
    syscribe_model::plant::register_backend(Box::new(FmiBackend))
}
