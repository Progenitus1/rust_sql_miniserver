use common::models::acid_sync::AcidSync;
use common::models::webserver_models::QueryResultData;
use query_parser::parser::query_parser::{parse, Query};

mod errors;
mod queries;
mod utils;

use errors::QueryError;
use queries::delete::process_delete_query;
use queries::index::{process_create_index_query, process_drop_index_query};
use queries::insert::process_insert_query;
use queries::select::process_select_query;
use queries::table::{process_create_table_query, process_drop_table_query};

type QueryResult = Result<QueryResultData, QueryError>;

pub fn process_query(query: &str, sync: AcidSync) -> QueryResult {
    match parse(query)? {
        Query::CreateTable {
            table_name,
            columns_definition,
        } => process_create_table_query(table_name, columns_definition, sync),
        Query::Insert {
            values,
            table_name,
            columns,
        } => process_insert_query(values, table_name, columns, sync),
        Query::Select {
            body,
            table_name,
            where_body,
        } => process_select_query(body, table_name, where_body, sync),
        Query::CreateIndex {
            column_name,
            table_name,
        } => process_create_index_query(column_name, table_name, sync),
        Query::DropIndex {
            column_name,
            table_name,
        } => process_drop_index_query(column_name, table_name, sync),
        Query::DropTable { table_name } => process_drop_table_query(table_name, sync),
        Query::Delete {
            table_name,
            where_body,
        } => process_delete_query(table_name, where_body, sync),
    }
}
