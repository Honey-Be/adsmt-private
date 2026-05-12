//! Uninterpreted functions theory (placeholder).
//!
//! v0.1 records asserted literals and detects direct polarity
//! contradictions (`p` and `¬p` for the same atom). Full congruence
//! closure arrives in v0.3 with E-graph and Nelson-Oppen equality
//! propagation.

use adsmt_cert::witness::{PoliteWitness, TheoryWitness};
use adsmt_core::{Term, Type};

use crate::trait_::{AssertResult, CheckResult, Literal, Theory};

#[derive(Default)]
pub struct Uf {
    pos: Vec<Term>,
    neg: Vec<Term>,
    conflict: Option<TheoryWitness>,
    scope_stack: Vec<(usize, usize)>,
}

impl Uf {
    pub fn new() -> Self { Self::default() }

    fn contains_alpha(set: &[Term], t: &Term) -> bool {
        set.iter().any(|x| x.alpha_eq(t))
    }
}

impl Theory for Uf {
    fn name(&self) -> &'static str { "UF" }

    fn handles_sort(&self, _: &Type) -> bool { true }

    fn assert(&mut self, lit: Literal) -> AssertResult {
        if lit.polarity {
            if Self::contains_alpha(&self.neg, &lit.term) {
                let w = TheoryWitness::Opaque {
                    kind: "UF".into(),
                    notes: format!("conflicting polarities on {}", lit.term),
                };
                self.conflict = Some(w.clone());
                return AssertResult::Conflict { witness: w };
            }
            if !Self::contains_alpha(&self.pos, &lit.term) {
                self.pos.push(lit.term);
            }
        } else {
            if Self::contains_alpha(&self.pos, &lit.term) {
                let w = TheoryWitness::Opaque {
                    kind: "UF".into(),
                    notes: format!("conflicting polarities on {}", lit.term),
                };
                self.conflict = Some(w.clone());
                return AssertResult::Conflict { witness: w };
            }
            if !Self::contains_alpha(&self.neg, &lit.term) {
                self.neg.push(lit.term);
            }
        }
        AssertResult::Accepted
    }

    fn check(&mut self) -> CheckResult {
        match &self.conflict {
            Some(w) => CheckResult::Unsat { witness: w.clone() },
            None => CheckResult::Sat,
        }
    }

    fn explain(&self) -> Option<TheoryWitness> { self.conflict.clone() }

    fn cardinality_witness(&self, sort: &Type) -> PoliteWitness {
        // UF is stably infinite over any sort it sees.
        PoliteWitness { sort: format!("{sort}"), upper_bound: None }
    }

    fn push(&mut self) {
        self.scope_stack.push((self.pos.len(), self.neg.len()));
    }

    fn pop(&mut self, levels: u32) {
        for _ in 0..levels {
            if let Some((p, n)) = self.scope_stack.pop() {
                self.pos.truncate(p);
                self.neg.truncate(n);
            }
        }
        // Re-evaluate conflict: if all conflicting literals were
        // popped, clear it. Conservative: just clear and let next
        // check re-discover.
        self.conflict = None;
        // Re-scan for direct contradictions.
        for t in self.pos.clone() {
            if Self::contains_alpha(&self.neg, &t) {
                self.conflict = Some(TheoryWitness::Opaque {
                    kind: "UF".into(),
                    notes: format!("conflicting polarities on {t}"),
                });
                break;
            }
        }
    }

    fn reset(&mut self) {
        self.pos.clear();
        self.neg.clear();
        self.conflict = None;
        self.scope_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adsmt_core::Term;

    #[test]
    fn empty_state_is_sat() {
        let mut uf = Uf::new();
        assert!(matches!(uf.check(), CheckResult::Sat));
    }

    #[test]
    fn detects_polarity_conflict() {
        let mut uf = Uf::new();
        let p = Term::var("p", Type::bool_());
        let pos = Literal::positive(p.clone()).unwrap();
        let neg = Literal::negative(p).unwrap();
        assert!(matches!(uf.assert(pos), AssertResult::Accepted));
        assert!(matches!(uf.assert(neg), AssertResult::Conflict { .. }));
        assert!(matches!(uf.check(), CheckResult::Unsat { .. }));
    }

    #[test]
    fn push_pop_restores_state() {
        let mut uf = Uf::new();
        let p = Term::var("p", Type::bool_());
        let q = Term::var("q", Type::bool_());
        uf.assert(Literal::positive(p).unwrap());
        uf.push();
        uf.assert(Literal::positive(q.clone()).unwrap());
        uf.assert(Literal::negative(q).unwrap());
        assert!(matches!(uf.check(), CheckResult::Unsat { .. }));
        uf.pop(1);
        assert!(matches!(uf.check(), CheckResult::Sat));
    }
}
