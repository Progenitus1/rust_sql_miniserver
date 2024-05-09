use common::models::db::Row;

use crate::table::table::Table;
use crate::table::{errors::PersistenceErrors};

use super::row::PersistenceRow;

pub struct RowsIterator {
    rows: Vec<Row>,
}

impl RowsIterator {
    pub fn from_table(table: &Table) -> Result<RowsIterator, PersistenceErrors> {
        let bytes = table.read_table_rows_bytes()?;
        let mut index = 0;
        let row_size = table.get_row_size();
        let mut rows = vec![];
        while index < bytes.len() {
            rows.push(Row::from_bytes(
                bytes[index..index + row_size].to_vec(),
                &table.columns,
            ));
            index += row_size;
        }
        Ok(RowsIterator { rows })
    }
}

impl Iterator for RowsIterator {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rows.get(0).is_some() {
            return Some(self.rows.remove(0));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::table::tests::{insert_data, insert_row};

    #[test]
    fn table_iterator() {
        let (table, row) = insert_data("Table2", false);
        let row1 = insert_row(
            &table,
            String::from("We will surely finish this project."),
            1,
        );
        let row2 = insert_row(&table, String::from("I am sure about it."), 10);
        let rows_iterator = RowsIterator::from_table(&table);
        assert!(rows_iterator.is_ok());
        let mut rows_iterator = rows_iterator.unwrap();
        assert_eq!(rows_iterator.next().unwrap(), row);
        assert_eq!(rows_iterator.next().unwrap(), row1);
        assert_eq!(rows_iterator.next().unwrap(), row2);
        assert!(rows_iterator.next().is_none());
        assert!(table.drop().is_ok())
    }
}
