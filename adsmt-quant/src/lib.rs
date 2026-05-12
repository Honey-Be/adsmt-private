//! Quantifier handling for adsmt.
//!
//! Four tiers escalate on deadlock: Miller-pattern HO E-matching,
//! conflict-based instantiation, bounded enumerative, then abductive
//! instantiation (handed to `adsmt-abduce`). Prenex normalization is a
//! preprocessing pass with explicit certificate steps.

pub mod prenex;
pub mod trigger;
pub mod ematch;
pub mod enumerate;
