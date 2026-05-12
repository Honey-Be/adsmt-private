//! HOL+HKT kernel for adsmt.
//!
//! Predicative rank-1 polymorphic HOL with first-order type-level
//! unification. The kernel exposes terms, types, kinds, and the 12
//! inference rules that define provability.

pub mod kind;
pub mod ty;
pub mod term;
pub mod theorem;
pub mod rule;
