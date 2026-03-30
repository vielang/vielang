//! Phase 38 — Asset Administration Shell (AAS) — Industry 4.0 standard.
//!
//! Implements the IDTA Asset Administration Shell structure:
//! Asset → AAS → Submodels (Nameplate, TechnicalData, OperationalData, Documentation).
//!
//! Reference: <https://www.plattform-i40.de/IP/Redaktion/EN/Standardization/AAS.html>

pub mod shell;
pub mod submodel;
pub mod nameplate;

pub use shell::*;
pub use submodel::*;
pub use nameplate::*;
