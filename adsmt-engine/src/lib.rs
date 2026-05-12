//! DPLL(T) engine and the public `Solver` API.
//!
//! Coordinates the SAT trail, polite theory combination, quantifier
//! tiers, and the abductive engine. Incremental push/pop is hard
//! (state-correct), with `abduce`/`promote`/`reject` layered above the
//! standard scope stack.

pub mod solver;
pub mod state;
pub mod dpllt;
pub mod result;
