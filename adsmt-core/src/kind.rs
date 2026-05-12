use std::fmt;
use std::sync::Arc;

/// Predicative rank-1 kinds: `Type`, `Type -> Type`, `(Type -> Type) -> Type`, ...
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Type,
    Arrow(Arc<Kind>, Arc<Kind>),
}

impl Kind {
    pub fn arrow(from: Kind, to: Kind) -> Kind {
        Kind::Arrow(Arc::new(from), Arc::new(to))
    }

    /// First-order kind of arity `n`: `Type -> Type -> ... -> Type` with `n` arrows.
    pub fn first_order(n: usize) -> Kind {
        let mut k = Kind::Type;
        for _ in 0..n {
            k = Kind::arrow(Kind::Type, k);
        }
        k
    }

    /// Number of arguments this kind expects.
    pub fn arity(&self) -> usize {
        match self {
            Kind::Type => 0,
            Kind::Arrow(_, k) => 1 + k.arity(),
        }
    }

    /// True iff every argument position has kind `Type` (i.e. no higher-order kinds).
    pub fn is_first_order(&self) -> bool {
        match self {
            Kind::Type => true,
            Kind::Arrow(a, b) => matches!(**a, Kind::Type) && b.is_first_order(),
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Type => write!(f, "Type"),
            Kind::Arrow(a, b) => {
                if matches!(**a, Kind::Arrow(..)) {
                    write!(f, "({a}) -> {b}")
                } else {
                    write!(f, "{a} -> {b}")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arity_of_type_is_zero() {
        assert_eq!(Kind::Type.arity(), 0);
    }

    #[test]
    fn first_order_arity_matches() {
        assert_eq!(Kind::first_order(0), Kind::Type);
        assert_eq!(Kind::first_order(1).arity(), 1);
        assert_eq!(Kind::first_order(3).arity(), 3);
    }

    #[test]
    fn first_order_predicate() {
        assert!(Kind::Type.is_first_order());
        assert!(Kind::first_order(2).is_first_order());
        let ho = Kind::arrow(Kind::first_order(1), Kind::Type);
        assert!(!ho.is_first_order());
    }

    #[test]
    fn display_forms() {
        assert_eq!(Kind::Type.to_string(), "Type");
        assert_eq!(Kind::first_order(1).to_string(), "Type -> Type");
        assert_eq!(Kind::first_order(2).to_string(), "Type -> Type -> Type");
        let ho = Kind::arrow(Kind::first_order(1), Kind::Type);
        assert_eq!(ho.to_string(), "(Type -> Type) -> Type");
    }
}
