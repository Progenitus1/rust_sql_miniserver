use std::collections::{HashMap, HashSet};

use common::models::{
    acid_sync::AcidSync,
    db::{Data, Row},
    webserver_models::QueryResultData,
};
use persistence::table::table::Table;
use query_parser::parser::lexer::LexerToken;

use crate::{errors::QueryError, utils::common::get_columns_definition_map, QueryResult};

pub fn process_insert_query(
    values: Vec<LexerToken>,
    table_name: String,
    columns: Vec<String>,
    sync: AcidSync,
) -> QueryResult {
    let rw_lock = sync.get_rw_lock(table_name.clone());
    let _x = rw_lock.write().unwrap();

    let table = Table::load(table_name.clone())?;
    let columns_def_map = get_columns_definition_map(&table);

    let columns = if columns.is_empty() {
        if values.len() != table.columns.len() {
            return Err(QueryError::IncorrectNumberOfValues(
                table.columns.len(),
                values.len(),
            ));
        }
        table
            .columns
            .iter()
            .map(|column| column.name.clone())
            .collect()
    } else {
        let mut column_usage: HashSet<String> = HashSet::new();
        for column_name in &columns {
            if columns_def_map.get(column_name).is_none() {
                return Err(QueryError::ColumnNotExists(
                    column_name.clone(),
                    table_name,
                ));
            }
            if column_usage.contains(column_name) {
                return Err(QueryError::DuplicateColumn(column_name.clone()));
            }
            column_usage.insert(column_name.clone());
        }
        columns
    };

    let data_map: HashMap<_, _> = columns
        .iter()
        .enumerate()
        .map(|(i, column)| (column.clone(), values[i].clone()))
        .collect();

    let insert_values: Vec<Data> = table
        .columns
        .iter()
        .map(|column| data_from_token(data_map.get(&column.name).unwrap_or(&LexerToken::Null)))
        .collect();

    // Check matching datatypes
    for (i, value) in insert_values.iter().enumerate() {
        if *value != Data::NULL && !value.is_valid_data_for_type(&table.columns[i].data_type) {
            return Err(QueryError::InvalidDataType(
                table.columns[i].name.clone(),
                table.columns[i].data_type.to_string(),
                value.to_type(),
            ));
        }
    }

    table.insert_row(&Row {
        values: insert_values,
    })?;

    Ok(QueryResultData {
        data: None,
        message: Some("1 row was succesfully inserted".to_string()),
    })
}

fn data_from_token(token: &LexerToken) -> Data {
    match token {
        LexerToken::NumberLiteral(number) => Data::INT(*number),
        LexerToken::StringLiteral(string) => Data::STRING(string.clone()),
        LexerToken::FloatNumberLiteral(f64) => Data::FLOAT(*f64),
        LexerToken::BoolLiteral(bool) => Data::BOOLEAN(*bool),
        _ => Data::NULL,
    }
}
