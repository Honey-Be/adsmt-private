//! Abductive engine for adsmt.
//!
//! SLD resolution with abducible insertion is glued to per-theory
//! `abduce` interfaces. Candidates are filtered for consistency,
//! minimized (subsumption then cardinality then depth), and returned as
//! ranked hypotheses with `explain` annotations threaded through.

pub mod abducible;
pub mod sld;
pub mod minimize;
pub mod rank;
pub mod workflow;
