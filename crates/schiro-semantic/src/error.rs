use std::fmt;

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub kind: ErrorKind,
    pub span: Option<(usize, usize)>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    UndefinedVariable(String),
    UndefinedType(String),
    UndefinedFunction(String),
    UndefinedTrait(String),
    DuplicateDefinition(String),
    TypeMismatch { expected: String, found: String },
    CannotMutate(String),
    WrongArity { name: String, expected: usize, found: usize },
    NotCallable(String),
    NotIndexable(String),
    NoSuchField { type_: String, field: String },
    TraitNotImplemented { type_: String, trait_: String },
    InvalidAssignmentTarget,
    UnusedVariable(String),
    UnreachableCode,
    Internal(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::UndefinedVariable(n) => write!(f, "undefined variable `{n}`"),
            ErrorKind::UndefinedType(n) => write!(f, "undefined type `{n}`"),
            ErrorKind::UndefinedFunction(n) => write!(f, "undefined function `{n}`"),
            ErrorKind::UndefinedTrait(n) => write!(f, "undefined trait `{n}`"),
            ErrorKind::DuplicateDefinition(n) => write!(f, "duplicate definition `{n}`"),
            ErrorKind::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected {expected}, found {found}")
            }
            ErrorKind::CannotMutate(n) => write!(f, "cannot mutate immutable variable `{n}`"),
            ErrorKind::WrongArity { name, expected, found } => {
                write!(f, "wrong number of arguments for `{name}`: expected {expected}, found {found}")
            }
            ErrorKind::NotCallable(n) => write!(f, "`{n}` is not callable"),
            ErrorKind::NotIndexable(n) => write!(f, "`{n}` is not indexable"),
            ErrorKind::NoSuchField { type_, field } => {
                write!(f, "no field `{field}` on type `{type_}`")
            }
            ErrorKind::TraitNotImplemented { type_, trait_ } => {
                write!(f, "type `{type_}` does not implement trait `{trait_}`")
            }
            ErrorKind::InvalidAssignmentTarget => write!(f, "invalid assignment target"),
            ErrorKind::UnusedVariable(n) => write!(f, "unused variable `{n}`"),
            ErrorKind::UnreachableCode => write!(f, "unreachable code"),
            ErrorKind::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}
