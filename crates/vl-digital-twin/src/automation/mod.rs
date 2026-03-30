//! Phase 39 — Automation Workflows: event-driven actions.
//!
//! When alarms, anomalies, or telemetry conditions trigger, automatically
//! execute actions: send RPC commands, update twin state, fire notifications.

pub mod workflow;
pub mod action;

pub use workflow::*;
pub use action::*;
