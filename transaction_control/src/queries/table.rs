use common::models::{
    acid_sync::AcidSync,
    db::{Column, DataType},
    webserver_models::QueryResultData,
};
use persistence::table::table::Table;

use crate::{errors::QueryError, utils, QueryResult};

pub fn process_create_table_query(
    table_name: String,
    columns_definition: Vec<(String, String)>,
    sync: AcidSync,
) -> QueryResult {
    let rw_lock = sync.get_rw_lock(table_name.clone());
    let _x = rw_lock.write().unwrap();

    if Table::load(table_name.clone()).is_ok() {
        return Err(QueryError::TableAlreadyExists(table_name));
    }

    let columns: Vec<Column> = columns_definition
        .into_iter()
        .map(|(name, data_type)| Column {
            name,
            data_type: from_string_to_data_type(data_type),
            is_indexed: false,
        })
        .collect();
    let cols_length = columns.len();
    let table = Table {
        name: table_name.clone(),
        columns,
    };

    table.create()?;

    utils::db_info::add_to_info_table(table_name, cols_length, sync)?;

    Ok(QueryResultData {
        data: None,
        message: Some(format!("Table {} created.", table.name)),
    })
}

pub fn process_drop_table_query(name: String, sync: AcidSync) -> QueryResult {
    let rw_lock = sync.get_rw_lock(name.clone());
    let _x = rw_lock.write().unwrap();

    let table = Table::load(name.clone())?;
    table.drop()?;
    utils::db_info::remove_from_info_table(name, sync)?;
    Ok(QueryResultData {
        data: None,
        message: Some(format!("Table {} dropped.", table.name)),
    })
}

fn from_string_to_data_type(data_type: String) -> DataType {
    match data_type.as_str() {
        "varchar" => DataType::STRING { size: 256 },
        "int" => DataType::INT,
        "boolean" => DataType::BOOLEAN,
        "float" => DataType::FLOAT,
        _ => unimplemented!(),
    }
}
