use std::collections::HashMap;

use common::models::{
    acid_sync::AcidSync,
    db::{Column, Data, DataType, Row},
    webserver_models::{QueryResultData, TableData},
};
use persistence::table::table::Table;
use query_parser::parser::{expression_tree::Node, lexer::LexerToken};

use crate::{
    errors::QueryError,
    utils::common::{get_columns_definition_map, get_rows_for_where_condition},
    QueryResult,
};

pub fn process_select_query(
    body: Vec<LexerToken>,
    table_name: String,
    where_body: Option<Node>,
    sync: AcidSync,
) -> QueryResult {
    let rw_lock = sync.get_rw_lock(table_name.clone());
    let _x = rw_lock.read().unwrap();

    let table = Table::load(table_name.clone())?;
    let columns_def_map = get_columns_definition_map(&table);

    let rows_numbers = get_rows_for_where_condition(&table, where_body)?;
    let mut rows = Vec::new();
    for row_number in rows_numbers {
        rows.push(table.seek_row(row_number)?);
    }

    let columns = get_projection_columns(body, table_name.clone(), table, &columns_def_map)?;

    let rows: Vec<Row> = rows
        .into_iter()
        .map(|data_row| project_row(data_row, &columns, &columns_def_map))
        .collect();

    let rows_count = rows.len();
    let data = TableData { columns, rows };
    Ok(QueryResultData {
        data: Some(data),
        message: Some(format!(
            "Retrieved {} rows from table {}.",
            rows_count, table_name
        )),
    })
}

fn get_projection_columns(
    body: Vec<LexerToken>,
    table_name: String,
    table: Table,
    columns_def_map: &HashMap<String, (usize, DataType)>,
) -> Result<Vec<Column>, QueryError> {
    let mut columns: Vec<Column> = Vec::new();
    for token in body {
        match token {
            LexerToken::Identifier(column) => {
                if !columns_def_map.contains_key(&column) {
                    return Err(QueryError::ColumnNotExists(column, table_name));
                }
                let data_type = columns_def_map.get(&column).unwrap().1;
                let column = Column {
                    name: column,
                    data_type,
                    is_indexed: false,
                };
                columns.push(column)
            }
            LexerToken::Star => columns.extend(table.columns.iter().cloned()),
            _ => unimplemented!(),
        }
    }
    Ok(columns)
}

fn project_row(
    row: Row,
    columns_res: &Vec<Column>,
    columns_def_map: &HashMap<String, (usize, DataType)>,
) -> Row {
    let mut row_projection: Vec<Data> = Vec::new();
    for column in columns_res {
        let index = columns_def_map.get(&column.name).unwrap().0;
        let value = row.values[index].clone();
        row_projection.push(value);
    }

    Row {
        values: row_projection,
    }
}
