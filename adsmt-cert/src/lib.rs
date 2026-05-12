//! Proof certificates for adsmt.
//!
//! Canonical S-expression format records every kernel rule application,
//! theory witness, type-class resolution, and abductive assumption. The
//! [`recorder`] module wraps the kernel rules in [`adsmt_core`] so that
//! invoking them also appends a step to a [`CertBuilder`].

pub mod canonical;
pub mod emit;
pub mod recorder;
pub mod witness;

pub use canonical::{CertBuilder, Certificate, Sequent, Step, StepBody, StepId};
pub use emit::emit_certificate;
pub use recorder::{ProofHandle, recorder as r};
pub use witness::{InstanceWitness, TheoryWitness};
