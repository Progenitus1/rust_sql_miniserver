use std::collections::HashMap;

use common::models::db::{Column, Data, DataType, Row};
use persistence::table::{row::PersistenceData, table::Table, table_iterator::RowsIterator};
use query_parser::parser::{
    expression_tree::Node,
    expression_tree_eval::{evaluate_binary_node, evaluate_node, NodeValue},
    lexer::LexerToken,
};

use crate::errors::QueryError;

pub fn get_rows_for_where_condition(
    table: &Table,
    where_body: Option<Node>,
) -> Result<Vec<u64>, QueryError> {
    let columns_def_map = get_columns_definition_map(table);
    let rows_iterator = RowsIterator::from_table(table)?;

    // prepare a vector of columns that are used in 'where body'
    let mut where_body_columns: Vec<&Column> = Vec::new();
    if let Some(where_node) = &where_body {
        let mut identifiers = Vec::new();
        where_node.collect_identifiers(&mut identifiers);
        // check if all identifers present in 'where body' are in column definitions
        for ident in identifiers {
            if !columns_def_map.contains_key(&ident) {
                return Err(QueryError::ColumnNotExists(ident, table.name.clone()));
            } else {
                let pos = columns_def_map.get(&ident).unwrap().0;
                where_body_columns.push(&table.columns[pos]);
            }
        }
    }

    // NOW proces only 'where body' in format of <WHERE><identifier><operator><value>
    let row_numbers = match &where_body {
        None => {
            // no where condition, return all rows
            (0..rows_iterator.count() as u64).collect()
        }
        // check if we support indexing for this query
        // currently, we should support only 'where column = value' queries
        Some(Node::Binary { left: _, op, right })
            if *op == LexerToken::CompareOp("=".into())
                && where_body_columns.len() == 1
                && where_body_columns[0].is_indexed =>
        {
            let mut result_rows = Vec::new();

            let index = table.get_index(where_body_columns[0])?;
            let searched_value = data_from_node(right)?; // we expect that the value is on the right side
            let index_row = index.rows.get(&searched_value.calculate_hash());
            if let Some(index_row) = index_row {
                for (data, row_number) in &index_row.values {
                    if *data == searched_value {
                        result_rows.push(*row_number);
                    }
                }
            }
            result_rows
        }
        // we cannot use index, let's apply the predicate on each row
        Some(node) => {
            let mut rows_i = Vec::new();
            for (i, row) in rows_iterator.enumerate() {
                if apply_row_predicate(&row, table, node)? {
                    rows_i.push(i as u64);
                }
            }
            rows_i
        }
    };

    Ok(row_numbers)
}

pub fn get_columns_definition_map(table: &Table) -> HashMap<String, (usize, DataType)> {
    table
        .columns
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, column)| (column.name, (index, column.data_type)))
        .collect()
}

fn apply_row_predicate(db_row: &Row, table: &Table, query_node: &Node) -> Result<bool, QueryError> {
    // combine the db_row and column definitions
    // so that it can be used in the query evaluator

    let mut identifier_map = HashMap::new();
    for (i, data_cell) in db_row.values.iter().enumerate() {
        let column = &table.columns[i];
        let data_value = match data_cell {
            Data::INT(number) => NodeValue::Int(*number),
            Data::STRING(string) => NodeValue::String(string.clone()),
            Data::NULL => NodeValue::Null,
            Data::BOOLEAN(bool) => NodeValue::Bool(*bool),
            Data::FLOAT(float) => NodeValue::Float(*float),
        };
        identifier_map.insert(column.name.clone(), data_value);
    }

    let bool_val = evaluate_binary_node(query_node, &identifier_map)?;
    Ok(bool_val)
}

fn data_from_node(node: &Node) -> Result<Data, QueryError> {
    let node_value = evaluate_node(node, &HashMap::new())?;

    Ok(match node_value {
        NodeValue::Int(number) => Data::INT(number),
        NodeValue::String(string) => Data::STRING(string),
        NodeValue::Bool(bool) => Data::BOOLEAN(bool),
        NodeValue::Float(float) => Data::FLOAT(float),
        NodeValue::Null => Data::NULL,
    })
}
