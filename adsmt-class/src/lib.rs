//! Type-class layer for adsmt.
//!
//! Relations elaborate to dictionary records over rank-1 polymorphic HOL.
//! Instances live in a hierarchical namespace with lexical scoping for
//! nested instances. Resolution is SLD with functional dependency
//! propagation; coherence is strict with an `overlap` opt-in.

pub mod relation;
pub mod instance;
pub mod resolve;
pub mod fundep;
