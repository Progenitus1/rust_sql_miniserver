use common::models::{acid_sync::AcidSync, webserver_models::QueryResultData};
use persistence::table::table::Table;
use query_parser::parser::expression_tree::Node;

use crate::{utils::common::get_rows_for_where_condition, QueryResult};

pub fn process_delete_query(
    table_name: String,
    where_body: Option<Node>,
    sync: AcidSync,
) -> QueryResult {
    let rw_lock = sync.get_rw_lock(table_name.clone());
    let _x = rw_lock.read().unwrap();

    let table = Table::load(table_name.clone())?;
    let row_numbers = get_rows_for_where_condition(&table, where_body)?;
    let rows_amount = row_numbers.len();
    table.delete_rows(row_numbers)?;

    Ok(QueryResultData {
        data: None,
        message: Some(format!(
            "Deleted {} rows from table {}.",
            rows_amount, table_name
        )),
    })
}
