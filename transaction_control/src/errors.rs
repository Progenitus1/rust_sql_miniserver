use std::io;

use persistence::table::errors::PersistenceErrors;
use query_parser::parser::errors::ParseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error(transparent)]
    IOTableAccess(#[from] io::Error),

    #[error(transparent)]
    ParseError(#[from] ParseError),

    #[error("column {0} does not exist in table {1}")]
    ColumnNotExists(String, String),

    #[error("column {0} can't be presented multiple times")]
    DuplicateColumn(String),

    #[error("table {0} already exist")]
    TableAlreadyExists(String),

    #[error("table has {0} columns but {1} values provided")]
    IncorrectNumberOfValues(usize, usize),

    #[error("column {0} has type {1} but the value with type {2} provided")]
    InvalidDataType(String, String, String),

    #[error(transparent)]
    Persistence(#[from] PersistenceErrors),
}
