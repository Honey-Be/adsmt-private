//! Proof certificates for adsmt.
//!
//! Canonical S-expression format records every kernel rule application,
//! theory witness, type-class resolution, and abductive assumption.
//! Multiple emit backends (Lean4, Alethe, JSON) consume this canonical form.

pub mod canonical;
pub mod emit;
pub mod witness;
