//! CNF decomposition for the v0.3 Boolean engine.
//!
//! [`flatten_to_clauses`] decomposes an asserted Boolean term into
//! a conjunction of clauses (each a disjunction of literals). Rules:
//!
//! - `(and p q)`              ⟶ flatten each side
//! - `(not (and p q))`        ⟶ flatten `(or (not p) (not q))`
//! - `(or p q)`               ⟶ single clause with each disjunct as literal
//! - `(not (or p q))`         ⟶ flatten each negated side as separate clauses
//! - `(=> p q)`               ⟶ `(or (not p) q)`
//! - `(not (not p))`          ⟶ flatten `p`
//! - `true`                   ⟶ empty clause set (vacuously true)
//! - `false`                  ⟶ a single empty clause (unsat)
//! - atomic literal `p` or `(not p)` ⟶ unit clause
//!
//! No Tseitin auxiliaries — nested OR-of-AND is flattened
//! syntactically. Complex nesting produces an exponential blow-up;
//! v0.5+ will switch to proper CNF transform with auxiliary
//! variables.

use adsmt_core::Term;

/// A literal: an atom paired with its polarity (true = positive).
#[derive(Clone, Debug)]
pub struct Lit {
    pub atom: Term,
    pub polarity: bool,
}

impl Lit {
    pub fn new(atom: Term, polarity: bool) -> Self { Self { atom, polarity } }
    pub fn pos(atom: Term) -> Self { Self::new(atom, true) }
    pub fn neg(atom: Term) -> Self { Self::new(atom, false) }
    pub fn negate(self) -> Self { Self { atom: self.atom, polarity: !self.polarity } }

    /// α-equivalence on atoms, polarity exact.
    pub fn matches(&self, other: &Lit) -> bool {
        self.polarity == other.polarity && self.atom.alpha_eq(&other.atom)
    }

    /// `p` vs `¬p`.
    pub fn is_negation_of(&self, other: &Lit) -> bool {
        self.polarity != other.polarity && self.atom.alpha_eq(&other.atom)
    }
}

/// A clause: disjunction of literals. Empty clause = false.
pub type Clause = Vec<Lit>;

/// Decompose `t` (asserted positively) into a conjunction of clauses.
/// Returns `Some(clauses)` if the structure is fully handled.
/// Compound structures we can't decompose syntactically (nested OR
/// of AND, etc.) return `None` — the engine treats the assertion as
/// opaque and reports Unknown if it can't be solved otherwise.
pub fn flatten_to_clauses(t: &Term) -> Option<Vec<Clause>> {
    flatten(t, true)
}

fn flatten(t: &Term, polarity: bool) -> Option<Vec<Clause>> {
    // (not P): flip polarity, recurse.
    if let Some(inner) = t.dest_not() {
        return flatten(&inner, !polarity);
    }
    // true / false handling under polarity.
    if t.is_true_const() {
        return Some(if polarity { Vec::new() } else { vec![Vec::new()] });
    }
    if t.is_false_const() {
        return Some(if polarity { vec![Vec::new()] } else { Vec::new() });
    }
    // Compound destructuring.
    match polarity {
        true => flatten_positive(t),
        false => flatten_negative(t),
    }
}

fn flatten_positive(t: &Term) -> Option<Vec<Clause>> {
    // (and p q): conjunction → both flattened independently.
    if let Some((p, q)) = t.dest_and() {
        let mut out = flatten(&p, true)?;
        out.extend(flatten(&q, true)?);
        return Some(out);
    }
    // (or p q): single clause containing flattened disjuncts as literals.
    if let Some((p, q)) = t.dest_or() {
        let mut lits = literals_of_disjunct(&p, true)?;
        lits.extend(literals_of_disjunct(&q, true)?);
        return Some(vec![lits]);
    }
    // (=> p q) === (or (not p) q)
    if let Some((p, q)) = t.dest_imp() {
        let mut lits = literals_of_disjunct(&p, false)?;
        lits.extend(literals_of_disjunct(&q, true)?);
        return Some(vec![lits]);
    }
    // Atomic literal.
    Some(vec![vec![Lit::pos(t.clone())]])
}

fn flatten_negative(t: &Term) -> Option<Vec<Clause>> {
    // De Morgan: (not (and p q)) → (or (not p) (not q))
    if let Some((p, q)) = t.dest_and() {
        let mut lits = literals_of_disjunct(&p, false)?;
        lits.extend(literals_of_disjunct(&q, false)?);
        return Some(vec![lits]);
    }
    // (not (or p q)) → (and (not p) (not q))
    if let Some((p, q)) = t.dest_or() {
        let mut out = flatten(&p, false)?;
        out.extend(flatten(&q, false)?);
        return Some(out);
    }
    // (not (=> p q)) === (and p (not q))
    if let Some((p, q)) = t.dest_imp() {
        let mut out = flatten(&p, true)?;
        out.extend(flatten(&q, false)?);
        return Some(out);
    }
    // Negative atom — unit clause.
    Some(vec![vec![Lit::neg(t.clone())]])
}

/// Extract a flat list of literals from a disjunct sub-expression.
/// Only handles `(or ...)`, `(not ...)`, and atoms; conjunctions
/// inside a disjunct are not decomposable without Tseitin auxiliaries.
fn literals_of_disjunct(t: &Term, polarity: bool) -> Option<Vec<Lit>> {
    if let Some(inner) = t.dest_not() {
        return literals_of_disjunct(&inner, !polarity);
    }
    if polarity {
        if let Some((p, q)) = t.dest_or() {
            let mut out = literals_of_disjunct(&p, true)?;
            out.extend(literals_of_disjunct(&q, true)?);
            return Some(out);
        }
        if let Some((p, q)) = t.dest_imp() {
            let mut out = literals_of_disjunct(&p, false)?;
            out.extend(literals_of_disjunct(&q, true)?);
            return Some(out);
        }
        if t.dest_and().is_some() {
            // (or ... (and ...) ...) — would need Tseitin aux var.
            return None;
        }
        if t.is_true_const() { return Some(vec![Lit::pos(Term::true_const())]); }
        if t.is_false_const() { return Some(Vec::new()); }
        Some(vec![Lit::pos(t.clone())])
    } else {
        if let Some((p, q)) = t.dest_and() {
            let mut out = literals_of_disjunct(&p, false)?;
            out.extend(literals_of_disjunct(&q, false)?);
            return Some(out);
        }
        if t.dest_or().is_some() || t.dest_imp().is_some() {
            return None;
        }
        Some(vec![Lit::neg(t.clone())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adsmt_core::Type;

    fn p() -> Term { Term::var("p", Type::bool_()) }
    fn q() -> Term { Term::var("q", Type::bool_()) }
    fn r() -> Term { Term::var("r", Type::bool_()) }

    #[test]
    fn atom_is_unit_clause() {
        let cs = flatten_to_clauses(&p()).unwrap();
        assert_eq!(cs.len(), 1);
        assert_eq!(cs[0].len(), 1);
        assert!(cs[0][0].polarity);
    }

    #[test]
    fn negation_flips_polarity() {
        let t = Term::mk_not(p()).unwrap();
        let cs = flatten_to_clauses(&t).unwrap();
        assert!(!cs[0][0].polarity);
    }

    #[test]
    fn conjunction_splits_into_clauses() {
        let t = Term::mk_and(p(), q()).unwrap();
        let cs = flatten_to_clauses(&t).unwrap();
        assert_eq!(cs.len(), 2);
    }

    #[test]
    fn disjunction_stays_single_clause() {
        let t = Term::mk_or(p(), q()).unwrap();
        let cs = flatten_to_clauses(&t).unwrap();
        assert_eq!(cs.len(), 1);
        assert_eq!(cs[0].len(), 2);
    }

    #[test]
    fn implication_rewrites_to_or_not() {
        let t = Term::mk_imp(p(), q()).unwrap();
        let cs = flatten_to_clauses(&t).unwrap();
        assert_eq!(cs.len(), 1);
        assert_eq!(cs[0].len(), 2);
        assert!(!cs[0][0].polarity); // not p
        assert!(cs[0][1].polarity);  // q
    }

    #[test]
    fn de_morgan_negated_and() {
        // ¬(p ∧ q) → (¬p ∨ ¬q) — one clause with two negative literals.
        let t = Term::mk_not(Term::mk_and(p(), q()).unwrap()).unwrap();
        let cs = flatten_to_clauses(&t).unwrap();
        assert_eq!(cs.len(), 1);
        assert!(cs[0].iter().all(|l| !l.polarity));
    }

    #[test]
    fn de_morgan_negated_or() {
        // ¬(p ∨ q) → (¬p) ∧ (¬q) — two unit clauses, both negative.
        let t = Term::mk_not(Term::mk_or(p(), q()).unwrap()).unwrap();
        let cs = flatten_to_clauses(&t).unwrap();
        assert_eq!(cs.len(), 2);
        assert!(cs.iter().all(|c| !c[0].polarity));
    }

    #[test]
    fn double_negation_cancels() {
        let t = Term::mk_not(Term::mk_not(p()).unwrap()).unwrap();
        let cs = flatten_to_clauses(&t).unwrap();
        assert_eq!(cs[0].len(), 1);
        assert!(cs[0][0].polarity);
    }

    #[test]
    fn variadic_and_via_nested_terms() {
        // (and p (and q r)) — right-fold from the parser
        let inner = Term::mk_and(q(), r()).unwrap();
        let t = Term::mk_and(p(), inner).unwrap();
        let cs = flatten_to_clauses(&t).unwrap();
        assert_eq!(cs.len(), 3); // one unit clause per atom
    }

    #[test]
    fn true_asserts_to_empty_clause_set() {
        let cs = flatten_to_clauses(&Term::true_const()).unwrap();
        assert!(cs.is_empty());
    }

    #[test]
    fn false_asserts_to_empty_clause_meaning_unsat() {
        let cs = flatten_to_clauses(&Term::false_const()).unwrap();
        assert_eq!(cs.len(), 1);
        assert!(cs[0].is_empty());
    }
}
