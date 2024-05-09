use thiserror::Error;

use super::{expression_tree_eval::NodeValue, lexer::LexerToken};

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("invalid char {0} found at pos {1}")]
    InvalidChar(char, usize),
    #[error("invalid char {0} in identifier {1}")]
    InvalidIdentifier(char, String),
    #[error("unfinished string literal {0}")]
    UnfinishedStringLiteral(String),
    #[error("unexpected query token - expected <{0}>, got {1:?}")]
    UnexpectedToken(String, LexerToken),
    #[error("unexpected query ending")]
    UnexpectedQueryEnding,
    #[error("unfinished parenthesis")]
    UnfinishedParenthesis,
    #[error("number of values in insert query does not match number of columns")]
    InsertQueryValuesMismatch,
    // maybe separate these errors..?
    #[error("invalid operator - expected <{0}>, got {1:?}")]
    InvalidOperator(String, LexerToken),
    #[error("invalid type - expected <{0}>, got {1:?}")]
    InvalidType(String, NodeValue),
    #[error("identifier {0} not found")]
    IdentifierNotFound(String),
}

pub type ParseResult<T> = Result<T, ParseError>;
