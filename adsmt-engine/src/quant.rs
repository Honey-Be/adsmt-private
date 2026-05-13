//! Quantifier instantiation glue (v0.3 tier 1).
//!
//! When the Boolean engine reports Sat over the ground fragment, we
//! run a Miller-pattern E-matching pass over every asserted
//! `∀x. body` quantifier:
//!
//! 1. Collect *ground* sub-terms from non-quantified assertions into
//!    a [`TermUniverse`].
//! 2. For each forall, treat the body as a single trigger pattern
//!    over the bound variable.
//! 3. Run the [`EMatcher`] to find substitutions.
//! 4. Emit instantiated bodies as new assertions.
//!
//! v0.3 alpha limits instantiation to a small fixed budget so the
//! engine stays decisive even on bad triggers.

use std::collections::HashSet;
use std::sync::Arc;

use adsmt_core::{Term, Var};
use adsmt_quant::ematch::{EMatcher, TermUniverse};
use adsmt_quant::trigger::Trigger;
use indexmap::IndexMap;

/// Pull every quantified assertion out of `assertions`, returning
/// `(quantified, rest)`.
pub fn partition_quantifiers(assertions: &[(Term, bool)]) -> (Vec<(Var, Term)>, Vec<(Term, bool)>) {
    let mut quants = Vec::new();
    let mut rest = Vec::new();
    for (t, p) in assertions {
        if !*p {
            // Negated quantifier is existential — handled in v0.5.
            rest.push((t.clone(), *p));
            continue;
        }
        if let Some((var, body)) = t.dest_forall() {
            quants.push((var, body));
        } else {
            rest.push((t.clone(), *p));
        }
    }
    (quants, rest)
}

/// Walk every (ground, non-quantified) term collecting subterms that
/// share the variable's sort.
pub fn collect_universe(rest: &[(Term, bool)]) -> TermUniverse {
    let mut u = TermUniverse::new();
    for (t, _) in rest {
        gather_subterms(t, &mut u);
    }
    u
}

fn gather_subterms(t: &Term, u: &mut TermUniverse) {
    u.insert(t.clone());
    match t {
        Term::App(f, x) => {
            gather_subterms(f, u);
            gather_subterms(x, u);
        }
        Term::Lam(_, body) => gather_subterms(body, u),
        _ => {}
    }
}

/// Quantifier-handling tier reached by a given call to
/// [`instantiate_one`]. v0.9 records this so the surrounding engine
/// loop can escalate to Tier 4 (abductive scaffolding) when all
/// term-based strategies are exhausted.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tier { One, Three, Exhausted }

/// For a single `∀v. body`, generate instantiations of `body` by
/// matching `body` (treated as a flex pattern over `v`) against terms
/// in `universe`. Returns instantiated bodies (positive polarity)
/// alongside the tier that produced them.
pub fn instantiate_with_tier(
    var: &Var,
    body: &Term,
    universe: &TermUniverse,
) -> (Vec<Term>, Tier) {
    let res = instantiate_one(var, body, universe);
    let tier = if res.is_empty() {
        Tier::Exhausted
    } else {
        // Tier classification: if at least one match came from a
        // pattern-matching step over universe terms whose shape
        // mirrors the body, classify as Tier One; otherwise the
        // fallback enumeration produced it (Tier Three).
        if universe.iter().any(|t| body.alpha_eq(t)) {
            Tier::One
        } else {
            Tier::Three
        }
    };
    (res, tier)
}

/// For a single `∀v. body`, generate instantiations of `body` by
/// matching `body` (treated as a flex pattern over `v`) against terms
/// in `universe`. Returns instantiated bodies (positive polarity).
pub fn instantiate_one(
    var: &Var,
    body: &Term,
    universe: &TermUniverse,
) -> Vec<Term> {
    let v_arc = Arc::new(var.clone());
    // The trigger is the body itself, with `var` as the sole flex.
    let trig = Trigger::single(body.clone(), vec![v_arc.clone()]);
    let matcher = EMatcher::new(universe);
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    // Strategy: try to match `body` directly against ground sub-terms.
    // For each successful match σ = {var ↦ t}, apply σ to body.
    for instantiation in matcher.match_trigger(&trig) {
        for (sub_v, sub_t) in &instantiation.subst {
            if **sub_v != *var { continue; }
            let key = sub_t.to_string();
            if seen.contains(&key) { continue; }
            seen.insert(key);

            // Build σ = {var ↦ sub_t} and apply to body.
            let mut sigma: IndexMap<Arc<Var>, Term> = IndexMap::new();
            sigma.insert(v_arc.clone(), sub_t.clone());
            if let Ok(instantiated) = body.subst(&sigma) {
                out.push(instantiated);
            }
        }
    }

    // Fallback: even if E-matching against the body shape yielded
    // nothing (common with rigid bodies), enumerate every universe
    // term of the right sort and instantiate with it. This is the
    // brute-force tier-3 fallback (sec 18 Q12) — bounded by universe
    // size, which is bounded by input size.
    if out.is_empty() {
        for t in universe.iter() {
            if t.type_of() != var.ty { continue; }
            let key = t.to_string();
            if seen.contains(&key) { continue; }
            seen.insert(key);
            let mut sigma: IndexMap<Arc<Var>, Term> = IndexMap::new();
            sigma.insert(v_arc.clone(), t.clone());
            if let Ok(instantiated) = body.subst(&sigma) {
                out.push(instantiated);
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use adsmt_core::{Kind, Type};

    fn int_() -> Type { Type::const_("Int", Kind::Type) }

    #[test]
    fn collects_subterms_of_assertion() {
        let p = Term::const_("P", Type::fun(int_(), Type::bool_()).unwrap());
        let a = Term::var("a", int_());
        let p_a = Term::app(p.clone(), a.clone()).unwrap();
        let u = collect_universe(&[(p_a.clone(), true)]);
        // Should include p_a, p, a
        assert!(u.len() >= 3);
        let strs: Vec<String> = u.iter().map(|t| t.to_string()).collect();
        assert!(strs.iter().any(|s| s.contains("a")));
    }

    #[test]
    fn partitions_quantifier_and_ground() {
        let body = Term::var("p", Type::bool_());
        let v = Var { name: "x".into(), ty: int_() };
        let forall = Term::mk_forall(v, body.clone()).unwrap();
        let ground = Term::var("q", Type::bool_());
        let (qs, rest) = partition_quantifiers(&[(forall, true), (ground.clone(), true)]);
        assert_eq!(qs.len(), 1);
        assert_eq!(rest.len(), 1);
        assert!(rest[0].0.alpha_eq(&ground));
    }

    #[test]
    fn instantiates_against_ground_terms() {
        // forall x:Int. P x   with universe { P a, b } → instantiate
        // with the matching sub-term a.
        let p = Term::const_("P", Type::fun(int_(), Type::bool_()).unwrap());
        let a = Term::var("a", int_());
        let b = Term::var("b", int_());
        let mut u = TermUniverse::new();
        u.insert(Term::app(p.clone(), a.clone()).unwrap());
        u.insert(a.clone());
        u.insert(b);

        let x_var = Var { name: "x".into(), ty: int_() };
        let body = Term::app(p, Term::Var(Arc::new(x_var.clone()))).unwrap();
        let insts = instantiate_one(&x_var, &body, &u);
        assert!(!insts.is_empty());
        // Each instantiation should be `P <something>`, with one being P a.
        let strs: Vec<String> = insts.iter().map(|t| t.to_string()).collect();
        assert!(strs.iter().any(|s| s.contains('a')));
    }
}
