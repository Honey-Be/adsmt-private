//! Theory of arrays — placeholder for v0.1.
//!
//! Full read-over-write reasoning lands in v0.3.

use adsmt_cert::witness::{PoliteWitness, TheoryWitness};
use adsmt_core::Type;

use crate::trait_::{AssertResult, CheckResult, Literal, Theory};

#[derive(Default)]
pub struct Arrays;

impl Theory for Arrays {
    fn name(&self) -> &'static str { "Arrays" }
    fn handles_sort(&self, _: &Type) -> bool { false }
    fn assert(&mut self, _: Literal) -> AssertResult { AssertResult::Ignored }
    fn check(&mut self) -> CheckResult { CheckResult::Sat }
    fn explain(&self) -> Option<TheoryWitness> { None }
    fn cardinality_witness(&self, sort: &Type) -> PoliteWitness {
        PoliteWitness { sort: format!("{sort}"), upper_bound: None }
    }
    fn reset(&mut self) {}
}
