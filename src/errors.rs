use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum TinyLangError {
    #[error("{0}")]
    ParserError(#[from] ParseError),
    #[error("{0}")]
    RuntimeError(#[from] RuntimeError),
}

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("{0}")]
    Generic(String),
    #[error("variable '{0}' is not defined")]
    VariableNotDefined(String),
    #[error("type mismatch: cannot apply this operator to the given types")]
    InvalidLangType,
    #[error("expected a number")]
    ExpectingNumber,
    #[error("'if' condition must evaluate to a bool")]
    ExpectingBool,
    #[error("'for' loop expected a Vec to iterate over, found {0}")]
    ExpectedVec(String),
    #[error("property access with '.' requires an Object, found {0}")]
    ExpectedObject(String),
    #[error("'{0}' is not callable")]
    NotAFunction(String),
}

#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("{0}")]
    Generic(String),
    #[error("{0}")]
    InvalidNode(String),
    #[error("'else' without a matching 'if'")]
    NoMatchingIf,
    #[error("'end' without a matching 'if' or 'for'")]
    NoMatchingFor,
    #[error("'if' or 'for' without a matching 'end'")]
    MissingEnd,
}
