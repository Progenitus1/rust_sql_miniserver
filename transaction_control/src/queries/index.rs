use common::models::{acid_sync::AcidSync, webserver_models::QueryResultData};
use persistence::table::table::Table;

use crate::{errors::QueryError, utils::common::get_columns_definition_map, QueryResult};

pub fn process_create_index_query(
    column_name: String,
    table_name: String,
    sync: AcidSync,
) -> QueryResult {
    let rw_lock = sync.get_rw_lock(table_name.clone());
    let _x = rw_lock.write().unwrap();

    let mut table = Table::load(table_name.clone())?;
    let columns_def_map = get_columns_definition_map(&table);

    if let Some((column_number, _)) = columns_def_map.get(&column_name) {
        table.add_index(*column_number)?;
    } else {
        return Err(QueryError::ColumnNotExists(
            column_name.clone(),
            table_name,
        ));
    }

    Ok(QueryResultData {
        data: None,
        message: Some(format!(
            "Index on column {} at table {} created succesfully.",
            column_name, table_name
        )),
    })
}

pub fn process_drop_index_query(
    column_name: String,
    table_name: String,
    sync: AcidSync,
) -> QueryResult {
    let rw_lock = sync.get_rw_lock(table_name.clone());
    let _x = rw_lock.write().unwrap();

    let mut table = Table::load(table_name.clone())?;
    let columns_def_map = get_columns_definition_map(&table);

    if let Some((column_number, _)) = columns_def_map.get(&column_name) {
        table.remove_index(*column_number)?;
    } else {
        return Err(QueryError::ColumnNotExists(
            column_name.clone(),
            table_name,
        ));
    }

    Ok(QueryResultData {
        data: None,
        message: Some(format!(
            "Index on column {} at table {} dropped succesfully.",
            column_name, table_name
        )),
    })
}
