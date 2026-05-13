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

/// Decision-splitting DPLL with bounded depth. Combines unit
/// propagation with backtracking case splits over unassigned atoms.
/// At depth 0 this is the same as [`unit_propagate`].
pub fn dpll(clauses: &[Clause], max_depth: usize) -> BoolResult {
    let assign = HashMap::new();
    dpll_rec(clauses, assign, max_depth)
}

fn dpll_rec(
    clauses: &[Clause],
    assign: HashMap<String, bool>,
    depth_budget: usize,
) -> BoolResult {
    // Run unit propagation to fixpoint, extending `assign`.
    let propagated = propagate_with(clauses, assign);
    let assign = match propagated {
        PropOutcome::Conflict => return BoolResult::Unsat,
        PropOutcome::Fixed(a) => a,
    };

    // All clauses satisfied?
    let mut all_sat = true;
    let mut decision_atom: Option<(String, &Lit)> = None;
    for clause in clauses {
        match evaluate_clause(clause, &assign) {
            ClauseEval::Satisfied => {}
            ClauseEval::Falsified => return BoolResult::Unsat,
            ClauseEval::Unit(_) => unreachable!("propagation drained all units"),
            ClauseEval::Open => {
                all_sat = false;
                // Pick a candidate atom to decide on: first
                // unassigned literal of the first open clause.
                if decision_atom.is_none() {
                    for lit in clause {
                        let key = atom_key(lit);
                        if !assign.contains_key(&key) {
                            decision_atom = Some((key, lit));
                            break;
                        }
                    }
                }
            }
        }
    }
    if all_sat { return BoolResult::Sat; }
    if depth_budget == 0 { return BoolResult::Unknown; }

    let (key, _lit) = match decision_atom {
        Some(d) => d,
        None => return BoolResult::Unknown,
    };

    // Try assigning true first.
    let mut a_true = assign.clone();
    a_true.insert(key.clone(), true);
    match dpll_rec(clauses, a_true, depth_budget - 1) {
        BoolResult::Sat => return BoolResult::Sat,
        BoolResult::Unsat => {} // try the other branch
        BoolResult::Unknown => return BoolResult::Unknown,
    }

    let mut a_false = assign;
    a_false.insert(key, false);
    dpll_rec(clauses, a_false, depth_budget - 1)
}

enum PropOutcome {
    Conflict,
    Fixed(HashMap<String, bool>),
}

fn propagate_with(
    clauses: &[Clause],
    mut assign: HashMap<String, bool>,
) -> PropOutcome {
    loop {
        let mut progress = false;
        for clause in clauses {
            match evaluate_clause(clause, &assign) {
                ClauseEval::Satisfied | ClauseEval::Open => continue,
                ClauseEval::Falsified => return PropOutcome::Conflict,
                ClauseEval::Unit(lit) => {
                    let key = atom_key(&lit);
                    if let Some(&v) = assign.get(&key) {
                        if v != lit.polarity { return PropOutcome::Conflict; }
                    } else {
                        assign.insert(key, lit.polarity);
                        progress = true;
                    }
                }
            }
        }
        if !progress { break; }
    }
    PropOutcome::Fixed(assign)
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
    fn dpll_decides_lone_disjunction() {
        // (p ∨ q) alone — propagation alone says Unknown; DPLL
        // tries p=true → satisfies clause → Sat.
        let cs = vec![vec![Lit::pos(p()), Lit::pos(q())]];
        assert_eq!(dpll(&cs, 4), BoolResult::Sat);
    }

    #[test]
    fn dpll_unsat_via_branching() {
        // (p ∨ q) ∧ (¬p ∨ q) ∧ (p ∨ ¬q) ∧ (¬p ∨ ¬q) — classic
        // pigeonhole-style 2-var unsat that requires both branches.
        let cs = vec![
            vec![Lit::pos(p()), Lit::pos(q())],
            vec![Lit::neg(p()), Lit::pos(q())],
            vec![Lit::pos(p()), Lit::neg(q())],
            vec![Lit::neg(p()), Lit::neg(q())],
        ];
        assert_eq!(dpll(&cs, 4), BoolResult::Unsat);
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
