//! Theory of (inductive and finite) datatypes — placeholder for v0.1.
//!
//! Full constructor disjointness / injectivity / acyclicity / case
//! split reasoning lands in v0.3. Finite enums supply explicit
//! politeness witnesses; inductive datatypes return ω.

use adsmt_cert::witness::{PoliteWitness, TheoryWitness};
use adsmt_core::Type;

use crate::trait_::{AssertResult, CheckResult, Literal, Theory};

#[derive(Default)]
pub struct Datatypes;

impl Theory for Datatypes {
    fn name(&self) -> &'static str { "Datatypes" }
    fn handles_sort(&self, _: &Type) -> bool { false }
    fn assert(&mut self, _: Literal) -> AssertResult { AssertResult::Ignored }
    fn check(&mut self) -> CheckResult { CheckResult::Sat }
    fn explain(&self) -> Option<TheoryWitness> { None }
    fn cardinality_witness(&self, sort: &Type) -> PoliteWitness {
        // Without metadata about which sort is finite-enum, default to ω.
        // Concrete impl will consult a datatype registry.
        PoliteWitness { sort: format!("{sort}"), upper_bound: None }
    }
    fn reset(&mut self) {}
}
