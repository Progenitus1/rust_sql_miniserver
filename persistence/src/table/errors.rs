use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceErrors {
    #[error("Table wasn't created.")]
    TableCreation(#[source] io::Error),
    #[error("Table wasn't dropped.")]
    TableDrop(#[source] io::Error),
    #[error("Insert wasn't successful.")]
    Insert(#[source] io::Error),
    #[error("Table couldn't be loaded.")]
    TableLoading(#[source] io::Error),
    #[error("Row with this number doesn't exist or there was io problem.")]
    RowSeeking(#[source] io::Error),
    #[error("Index was unable to be refreshed.")]
    IndexRefresh(#[source] io::Error),
    #[error("Index wasn't loaded properly.")]
    IndexLoading(#[source] io::Error),
    #[error("Index wasn't loaded properly.")]
    IndexCreating(),
    #[error("Row wasn't deleted properly.")]
    RowDeletion(#[source] io::Error),
}
