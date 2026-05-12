use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum KernelError {
    #[error("kind mismatch: expected {expected}, found {found}")]
    KindMismatch { expected: String, found: String },

    #[error("cannot apply kind {fun_kind} to {arg_kind}")]
    InvalidKindApplication { fun_kind: String, arg_kind: String },

    #[error("type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("not a function type: {0}")]
    NotFunctionType(String),

    #[error("conclusion is not an equation: {0}")]
    NotEquation(String),

    #[error("TRANS middle terms differ: {lhs} vs {rhs}")]
    TransMismatch { lhs: String, rhs: String },

    #[error("ABS: bound variable {0} appears free in hypotheses")]
    AbsFreeInHyps(String),

    #[error("EQ_MP: conclusion is not an iff: {0}")]
    NotIff(String),

    #[error("EQ_MP: antecedent mismatch: expected {expected}, found {found}")]
    EqMpMismatch { expected: String, found: String },

    #[error("BETA: not a lambda redex: {0}")]
    NotBetaRedex(String),
}

pub type KernelResult<T> = Result<T, KernelError>;
