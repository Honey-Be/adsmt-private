//! Parsers and emitters for adsmt input surfaces.
//!
//! SMT-LIB v2 support is phased by theory; the lu-kb surface follows
//! logicutils' indentation-sensitive grammar with kind inference and
//! `F(_)` slot sugar.

pub mod smtlib;
pub mod kb;
