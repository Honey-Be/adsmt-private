//! Linear arithmetic (LIA / LRA) — placeholder for v0.1.
//!
//! Full Simplex tableau + Fourier-Motzkin abduction lands in v0.3.

use adsmt_cert::witness::{PoliteWitness, TheoryWitness};
use adsmt_core::Type;

use crate::trait_::{AssertResult, CheckResult, Literal, Theory};

pub struct LinArith {
    name_: &'static str,
}

impl LinArith {
    pub fn lia() -> Self { Self { name_: "LIA" } }
    pub fn lra() -> Self { Self { name_: "LRA" } }
}

impl Theory for LinArith {
    fn name(&self) -> &'static str { self.name_ }
    fn handles_sort(&self, _: &Type) -> bool { false }
    fn assert(&mut self, _: Literal) -> AssertResult { AssertResult::Ignored }
    fn check(&mut self) -> CheckResult { CheckResult::Sat }
    fn explain(&self) -> Option<TheoryWitness> { None }
    fn cardinality_witness(&self, sort: &Type) -> PoliteWitness {
        PoliteWitness { sort: format!("{sort}"), upper_bound: None }
    }
    fn reset(&mut self) {}
}
