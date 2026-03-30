//! Phase 36 — DTDL (Digital Twins Definition Language) v3 support.
//!
//! Implements a subset of Microsoft's DTDL v3 standard for defining digital twin
//! models. This gives VíeLang compatibility with Azure Digital Twins and the broader
//! industrial ecosystem.
//!
//! Reference: <https://azure.github.io/opendigitaltwins-dtdl/DTDL/v3/DTDL.v3.html>

pub mod model;
pub mod instance;
pub mod parser;
pub mod registry;

pub use model::*;
pub use instance::*;
pub use parser::*;
pub use registry::*;
