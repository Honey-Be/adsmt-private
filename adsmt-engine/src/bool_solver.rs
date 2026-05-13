//! Boolean reasoning over CNF clauses.
//!
//! v0.3 alpha: unit propagation only. If propagation derives an
//! empty clause → Unsat. If every clause is satisfied → Sat.
//! Otherwise → Unknown (decision splitting is gated for v0.5
//! when proper CDCL lands).

use std::collections::HashMap;

use crate::cnf::{Clause, Lit};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoolResult {
    Sat,
    Unsat,
    /// Propagation reached a fixpoint but some clauses are still open.
    Unknown,
}

/// Run unit propagation on `clauses`.
pub fn unit_propagate(clauses: &[Clause]) -> BoolResult {
    // Atom assignment: atom-as-string-of-display → polarity.
    // Using Display string keeps atoms with α-equivalent content unified.
    let mut assign: HashMap<String, bool> = HashMap::new();

    // Loop until fixpoint.
    loop {
        let mut progress = false;
        for clause in clauses {
            match evaluate_clause(clause, &assign) {
                ClauseEval::Satisfied => continue,
                ClauseEval::Falsified => return BoolResult::Unsat,
                ClauseEval::Unit(lit) => {
                    let key = atom_key(&lit);
                    if let Some(&existing) = assign.get(&key) {
                        if existing != lit.polarity {
                            return BoolResult::Unsat;
                        }
                    } else {
                        assign.insert(key, lit.polarity);
                        progress = true;
                    }
                }
                ClauseEval::Open => continue,
            }
        }
        if !progress {
            break;
        }
    }

    // Final pass: are all clauses satisfied by the assignment?
    let mut all_sat = true;
    for clause in clauses {
        match evaluate_clause(clause, &assign) {
            ClauseEval::Satisfied => {}
            ClauseEval::Falsified => return BoolResult::Unsat,
            _ => all_sat = false,
        }
    }
    if all_sat { BoolResult::Sat } else { BoolResult::Unknown }
}

fn atom_key(lit: &Lit) -> String {
    lit.atom.to_string()
}

enum ClauseEval {
    Satisfied,
    Falsified,
    Unit(Lit),
    Open,
}

fn evaluate_clause(clause: &Clause, assign: &HashMap<String, bool>) -> ClauseEval {
    let mut unassigned: Vec<&Lit> = Vec::new();
    for lit in clause {
        let key = atom_key(lit);
        match assign.get(&key) {
            Some(&v) if v == lit.polarity => return ClauseEval::Satisfied,
            Some(_) => continue, // assigned to false under this literal
            None => unassigned.push(lit),
        }
    }
    match unassigned.len() {
        0 => ClauseEval::Falsified,
        1 => ClauseEval::Unit(unassigned[0].clone()),
        _ => ClauseEval::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adsmt_core::{Term, Type};

    fn p() -> Term { Term::var("p", Type::bool_()) }
    fn q() -> Term { Term::var("q", Type::bool_()) }

    #[test]
    fn empty_clauses_is_sat() {
        assert_eq!(unit_propagate(&[]), BoolResult::Sat);
    }

    #[test]
    fn empty_clause_is_unsat() {
        let cs: Vec<Clause> = vec![vec![]];
        assert_eq!(unit_propagate(&cs), BoolResult::Unsat);
    }

    #[test]
    fn polarity_contradiction_via_units() {
        let cs = vec![vec![Lit::pos(p())], vec![Lit::neg(p())]];
        assert_eq!(unit_propagate(&cs), BoolResult::Unsat);
    }

    #[test]
    fn unit_propagation_satisfies_clause() {
        // p ∧ (p ∨ q) → sat
        let cs = vec![
            vec![Lit::pos(p())],
            vec![Lit::pos(p()), Lit::pos(q())],
        ];
        assert_eq!(unit_propagate(&cs), BoolResult::Sat);
    }

    #[test]
    fn implication_with_premise_forces_conclusion() {
        // (¬p ∨ q) ∧ p → forces q; sat overall.
        let cs = vec![
            vec![Lit::neg(p()), Lit::pos(q())],
            vec![Lit::pos(p())],
        ];
        assert_eq!(unit_propagate(&cs), BoolResult::Sat);
    }

    #[test]
    fn pure_disjunction_alone_is_unknown() {
        // (p ∨ q) — no way to decide without branching.
        let cs = vec![vec![Lit::pos(p()), Lit::pos(q())]];
        assert_eq!(unit_propagate(&cs), BoolResult::Unknown);
    }

    #[test]
    fn modus_tollens_chain() {
        // p, p→q, q→r, ¬r → unsat (propagation closes it)
        let r = Term::var("r", Type::bool_());
        let cs = vec![
            vec![Lit::pos(p())],
            vec![Lit::neg(p()), Lit::pos(q())],
            vec![Lit::neg(q()), Lit::pos(r.clone())],
            vec![Lit::neg(r)],
        ];
        assert_eq!(unit_propagate(&cs), BoolResult::Unsat);
    }
}
