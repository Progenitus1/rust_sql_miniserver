use common::models::{
    acid_sync::AcidSync,
    db::{Column, DataType},
};
use persistence::table::table::Table;

use crate::errors::QueryError;

static TABLES_INFO_NAME: &str = "all_tables";

pub fn add_to_info_table(table_name: String, cols_count: usize, sync: AcidSync) -> Result<(), QueryError> {
    let table_exists = Table::load(TABLES_INFO_NAME.to_string());

    if table_exists.is_err() {
        create_info_table()?;
    };

    let query = format!(
        "INSERT INTO {} VALUES ('{}', {})",
        TABLES_INFO_NAME, table_name, cols_count
    );
    crate::process_query(query.as_str(), sync)?;

    Ok(())
}

pub fn remove_from_info_table(table_name: String, sync: AcidSync) -> Result<(), QueryError> {
    let query = format!(
        "DELETE FROM {} WHERE table_name = '{}'",
        TABLES_INFO_NAME, table_name
    );
    crate::process_query(query.as_str(), sync)?;

    Ok(())
}

fn create_info_table() -> Result<Table, QueryError> {
    let table = Table {
        name: TABLES_INFO_NAME.to_string(),
        columns: vec![
            Column {
                name: "table_name".to_string(),
                data_type: DataType::STRING { size: 256 },
                is_indexed: false,
            },
            Column {
                name: "columns_count".to_string(),
                data_type: DataType::INT,
                is_indexed: false,
            },
        ],
    };

    table.create()?;
    Ok(table)
}
