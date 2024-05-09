use common::models::db::{Column, Row};

use crate::table::index::{Index, IndexRow};
use crate::table::{errors::PersistenceErrors,table_iterator};
use std::collections::HashMap;
use std::fs::{remove_file, write, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem;

use super::column::PersistenceColumn;
use super::row::{PersistenceData, PersistenceRow};

#[derive(Eq, PartialEq, Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

impl Table {
    pub fn add_index(&mut self, column_index: usize) -> Result<(), PersistenceErrors> {
        let column = self.columns.get(column_index);
        if column.is_none() {
            return Err(PersistenceErrors::IndexCreating());
        }
        let column = column.unwrap();
        let file_name = self.get_index_file_name(column);
        let table_iterator: Vec<(usize, Row)> = table_iterator::RowsIterator::from_table(self)?
            .enumerate()
            .collect();
        let column = self.columns.get_mut(column_index).expect("We already checked it is not none");
        column.is_indexed = true;
        let mut index_row_map = HashMap::new();
        for (row_number, row) in table_iterator.iter() {
            let option = row.values.get(column_index);
            let data = option.unwrap();
            index_row_map
                .entry(data.calculate_hash())
                .or_insert(vec![])
                .push((data.clone(), *row_number as u64));
        }
        let mut index = Index { rows: HashMap::new() };
        for (hash, values) in &mut index_row_map {
            index.rows.insert(*hash, IndexRow {
                hash: *hash,
                values: mem::take(values),
            });
        }
        index.write_index_to_file(file_name, column)?;
        self.write_table_header()?;
        Ok(())
    }

    pub fn remove_index(&mut self, column_index: usize) -> Result<(), PersistenceErrors> {
        let column = self.columns.get_mut(column_index);
        if column.is_none() {
            return Err(PersistenceErrors::IndexCreating());
        }
        let column = column.expect("We already checked it is not none");
        column.is_indexed = false;
        let column = self.columns.get(column_index).expect("We already checked it is not none");
        self.write_table_header()?;
        remove_file(self.get_index_file_name(column))
            .map_err(PersistenceErrors::TableDrop)?;
        Ok(())
    }

    pub fn seek_row(&self, row_number: u64) -> Result<Row, PersistenceErrors> {
        let mut rows_file =
            File::open(self.table_rows_name()).map_err(PersistenceErrors::RowSeeking)?;
        let row_size = self.get_row_size();
        rows_file
            .seek(SeekFrom::Start(row_number * (row_size as u64)))
            .map_err(PersistenceErrors::RowSeeking)?;
        let mut bytes: Vec<u8> = vec![0; row_size];
        let mut rows_file = rows_file.take(row_size as u64);
        rows_file
            .read_exact(&mut bytes)
            .map_err(PersistenceErrors::RowSeeking)?;
        Ok(Row::from_bytes(bytes, &self.columns))
    }

    pub fn get_row_size(&self) -> usize {
        let mut row_size = 0;
        for column in &self.columns {
            row_size += column.size();
        }
        row_size
    }

    pub fn create(&self) -> Result<(), PersistenceErrors> {
        self.write_table_header()?;
        write(self.table_rows_name(), []).map_err(PersistenceErrors::TableCreation)?;
        for column in &self.columns {
            if column.is_indexed {
                write(self.get_index_file_name(column), [])
                    .map_err(PersistenceErrors::TableCreation)?;
            }
        }
        Result::Ok(())
    }

    fn write_table_header(&self) -> Result<(), PersistenceErrors> {
        write(&self.name, self.to_bytes()).map_err(PersistenceErrors::TableCreation)?;
        Ok(())
    }

    pub(crate) fn table_rows_name(&self) -> String {
        self.name.clone() + "_rows"
    }

    pub fn drop(&self) -> Result<(), PersistenceErrors> {
        remove_file(&self.name).map_err(PersistenceErrors::TableDrop)?;
        remove_file(self.table_rows_name()).map_err(PersistenceErrors::TableDrop)?;
        for column in &self.columns {
            if column.is_indexed {
                remove_file(self.get_index_file_name(column))
                    .map_err(PersistenceErrors::TableDrop)?;
            }
        }
        Result::Ok(())
    }

    pub fn insert_row(&self, row: &Row) -> Result<(), PersistenceErrors> {
        let mut rows_file = OpenOptions::new()
            .append(true)
            .open(self.table_rows_name())
            .map_err(PersistenceErrors::Insert)?;
        rows_file
            .write_all(&row.to_bytes(&self.columns))
            .map_err(PersistenceErrors::Insert)?;
        self.generate_indexes()?;
        Result::Ok(())
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut columns_bytes: Vec<Vec<u8>> = vec![];
        for column in &self.columns {
            columns_bytes.push(column.to_bytes());
        }

        [
            (self.name.len() as u32).to_be_bytes().to_vec(),
            self.name.as_bytes().to_vec(),
            columns_bytes.concat(),
        ]
        .concat()
    }

    pub fn load(name: String) -> Result<Table, PersistenceErrors> {
        let bytes = std::fs::read(name).map_err(PersistenceErrors::TableLoading)?;
        Result::Ok(Table::from_bytes(bytes))
    }

    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Table {
        let name_size = get_size(&bytes);
        let name = match String::from_utf8(bytes[4usize..4usize + name_size].to_owned()) {
            Ok(string) => string,
            Err(_) => {
                panic!()
            }
        };
        let mut columns: Vec<Column> = vec![];
        let mut column_begging: usize = 4usize + name_size as usize;
        while column_begging < bytes.len() {
            let column_size = get_size(&bytes[column_begging..column_begging + 4usize]) + 13usize;
            columns.push(Column::from_bytes(
                bytes[column_begging..column_begging + column_size].to_vec(),
            ));
            column_begging += column_size;
        }

        Table { name, columns }
    }

    pub fn generate_indexes(&self) -> Result<(), PersistenceErrors> {
        let mut indexed_columns = vec![];
        for (index, column) in self.columns.iter().enumerate() {
            if column.is_indexed {
                indexed_columns.push((index, column, HashMap::new()));
            }
        }
        let table_iterator: Vec<(usize, Row)> = table_iterator::RowsIterator::from_table(self)?
            .enumerate()
            .collect();
        for (row_number, row) in table_iterator.iter() {
            for (column_number, _column, index_rows_map) in &mut indexed_columns {
                let option = row.values.get(*column_number);
                let data = option.unwrap();
                index_rows_map
                    .entry(data.calculate_hash())
                    .or_insert(vec![])
                    .push((data.clone(), *row_number as u64));
            }
        }
        for (_column_number, column, index_rows_map) in &mut indexed_columns {
            let mut index = Index { rows: HashMap::new() };
            for (hash, values) in index_rows_map {
                index.rows.insert(*hash, IndexRow {
                    hash: *hash,
                    values: mem::take(values),
                });
            }
            index.write_index_to_file(self.get_index_file_name(column), column)?;
        }

        Ok(())
    }

    fn get_index_file_name(&self, column: &Column) -> String {
        self.name.clone() + column.name.clone().as_str() + "_index"
    }

    pub(crate) fn read_table_rows_bytes(&self) -> Result<Vec<u8>, PersistenceErrors> {
        std::fs::read(self.table_rows_name()).map_err(PersistenceErrors::TableLoading)
    }

    pub fn delete_rows(&self, row_numbers: Vec<u64>) -> Result<(), PersistenceErrors> {
        let rows_bytes = self.read_table_rows_bytes()?;
        let row_size = self.get_row_size();
        let row_count = rows_bytes.len() / row_size;
        let mut new_rows_bytes = vec![];
        let mut rows_bytes_iterator = rows_bytes.iter();
        for row_number in 0..row_count {
            if row_numbers.contains(&(row_number as u64)) {
                for _ in 0..row_size {
                    rows_bytes_iterator
                        .next()
                        .expect("Element you are try to delete doesn't exist.");
                }
            } else {
                for _ in 0..row_size {
                    new_rows_bytes.push(
                        *rows_bytes_iterator
                            .next()
                            .expect("There is different number of rows then you thought."),
                    );
                }
            }
        }
        write(self.table_rows_name(), new_rows_bytes).map_err(PersistenceErrors::RowDeletion)?;
        self.generate_indexes()?;
        Ok(())
    }

    pub fn get_index(&self, column: &Column) -> Result<Index, PersistenceErrors> {
        let string = self.get_index_file_name(column);
        Index::load(string, &column)
    }

}

fn get_size(bytes: &[u8]) -> usize {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
}

#[cfg(test)]
pub mod tests {
    use common::models::db::{DataType, Data};

    use super::*;
    use crate::table::index;
    use std::path::Path;

    #[test]
    fn table_to_and_from_bytes() {
        let string_data_type = DataType::STRING { size: 255 };
        let column_name = Column {
            name: String::from("Name"),
            data_type: string_data_type,
            is_indexed: false,
        };
        let column_id = Column {
            name: String::from("Id"),
            data_type: DataType::INT,
            is_indexed: false,
        };
        let table = Table {
            name: String::from("Table"),
            columns: vec![column_name, column_id],
        };
        let table_from_bytes = Table::from_bytes(table.to_bytes());

        assert_eq!(table.name, table_from_bytes.name);
        assert_eq!(table_from_bytes.columns.len(), 2usize);
        let column_name_from_bytes = &table_from_bytes.columns[0];
        assert_eq!(column_name_from_bytes.name, String::from("Name"));
        match column_name_from_bytes.data_type {
            DataType::STRING { size } => {
                assert_eq!(size, 255);
            }
            _ => {
                panic!();
            }
        }
        let column_id_from_bytes = &table_from_bytes.columns[1];
        assert_eq!(column_id_from_bytes.name, String::from("Id"));
        match column_id_from_bytes.data_type {
            DataType::INT => {
                assert_eq!(1, 1);
            }
            _ => {
                panic!();
            }
        }
    }

    #[test]
    fn table_create() {
        let table = create_table("Table", false);
        assert!(table.create().is_ok());
        let table_path = Path::new(&table.name);
        assert!(table_path.exists());
        let table_rows_path = table.table_rows_name();
        let table_rows_path = Path::new(&table_rows_path);
        assert!(table_rows_path.exists());
        assert_eq!(std::fs::read(table_path).unwrap(), table.to_bytes());
        assert!(std::fs::read(table_rows_path).unwrap().is_empty());
        assert!(table.drop().is_ok())
    }

    fn create_table(name: &str, indexed: bool) -> Table {
        let string_data_type = DataType::STRING { size: 255 };
        let column_name = Column {
            name: String::from("Name"),
            data_type: string_data_type,
            is_indexed: false,
        };
        let column_id = Column {
            name: String::from("Id"),
            data_type: DataType::INT,
            is_indexed: indexed,
        };
        let table = Table {
            name: String::from(name),
            columns: vec![column_name, column_id],
        };
        table
    }

    #[test]
    fn table_indexes() {
        let (table, _row) = insert_data("Table7", true);
        insert_row(
            &table,
            String::from("We will surely finish this project."),
            1,
        );
        insert_row(&table, String::from("I am sure about it."), 10);
        let column_id = Column {
            name: String::from("Id"),
            data_type: DataType::INT,
            is_indexed: true,
        };
        let string = table.get_index_file_name(&column_id);
        let table_rows_path = Path::new(&string);
        assert!(table_rows_path.exists());
        let loaded_index = index::Index::load(string, &column_id).unwrap();
        assert!(table.drop().is_ok());
        let int_data = Data::INT(8);
        let int_data2 = Data::INT(10);
        let int_data3 = Data::INT(1);
        let index_row1 = IndexRow {
            hash: int_data.calculate_hash(),
            values: vec![(int_data, 0)],
        };
        let index_row2 = IndexRow {
            hash: int_data2.calculate_hash(),
            values: vec![(int_data2, 2)],
        };
        let index_row3 = IndexRow {
            hash: int_data3.calculate_hash(),
            values: vec![(int_data3, 1)],
        };

        assert!(matches!(loaded_index.rows.get(&index_row1.hash), Some(row) if *row == index_row1));
        assert!(matches!(loaded_index.rows.get(&index_row2.hash), Some(row) if *row == index_row2));
        assert!(matches!(loaded_index.rows.get(&index_row3.hash), Some(row) if *row == index_row3));
    }

    #[test]
    fn table_indexes_added_after_creation() {
        let (mut table, _row) = insert_data("Table9", false);
        insert_row(
            &table,
            String::from("We will surely finish this project."),
            1,
        );
        insert_row(&table, String::from("I am sure about it."), 10);
        let index_was_created_successfully = table.add_index(1).is_ok();
        let column_id = Column {
            name: String::from("Id"),
            data_type: DataType::INT,
            is_indexed: true,
        };
        let loaded_table = Table::load(String::from("Table9")).unwrap();
        assert_eq!(loaded_table, table);
        let string = table.get_index_file_name(&column_id);
        let table_rows_path = Path::new(&string);
        assert!(table_rows_path.exists());
        let loaded_index = index::Index::load(string, &column_id).unwrap();
        assert!(table.drop().is_ok());
        assert!(index_was_created_successfully);
        let int_data = Data::INT(8);
        let int_data2 = Data::INT(10);
        let int_data3 = Data::INT(1);
        let index_row1 = IndexRow {
            hash: int_data.calculate_hash(),
            values: vec![(int_data, 0)],
        };
        let index_row2 = IndexRow {
            hash: int_data2.calculate_hash(),
            values: vec![(int_data2, 2)],
        };
        let index_row3 = IndexRow {
            hash: int_data3.calculate_hash(),
            values: vec![(int_data3, 1)],
        };

        assert!(matches!(loaded_index.rows.get(&index_row1.hash), Some(row) if *row == index_row1));
        assert!(matches!(loaded_index.rows.get(&index_row2.hash), Some(row) if *row == index_row2));
        assert!(matches!(loaded_index.rows.get(&index_row3.hash), Some(row) if *row == index_row3));
    }

    #[test]
    fn table_indexes_deletion() {
        let (mut table, _row) = insert_data("Table10", true);
        insert_row(
            &table,
            String::from("We will surely finish this project."),
            1,
        );
        assert!(table.remove_index(1).is_ok());
        let loaded_table = Table::load(String::from("Table10")).unwrap();
        assert_eq!(table, loaded_table);
        assert!(table.drop().is_ok());
    }

    #[test]
    fn table_load() {
        let name = String::from("Table4");
        let table = create_table(name.clone().as_str(), false);
        assert!(table.create().is_ok());
        let loaded_table = Table::load(name).unwrap();
        assert!(table.drop().is_ok());
        assert_eq!(loaded_table, table);
    }

    #[test]
    fn table_insert() {
        let (table, row) = insert_data("Table1", false);
        let table_rows_path = table.table_rows_name();
        let table_rows_path = Path::new(&table_rows_path);
        assert_eq!(
            std::fs::read(table_rows_path).unwrap(),
            row.to_bytes(&table.columns)
        );
        assert!(table.drop().is_ok())
    }

    #[test]
    fn row_seeking() {
        let (table, row) = insert_data("Table6", false);
        let row1 = insert_row(
            &table,
            String::from("We will surely finish this project."),
            1,
        );
        let row2 = insert_row(&table, String::from("I am sure about it."), 10);
        assert_eq!(table.seek_row(0).unwrap(), row);
        assert_eq!(table.seek_row(1).unwrap(), row1);
        assert_eq!(table.seek_row(2).unwrap(), row2);
        assert!(table.seek_row(3).is_err());
        assert!(table.drop().is_ok())
    }

    #[test]
    fn delete_row() {
        let (table, row) = insert_data("Table8", false);
        insert_row(
            &table,
            String::from("We will surely finish this project."),
            1,
        );
        insert_row(&table, String::from("I am sure about it."), 10);
        let row3 = insert_row(&table, String::from("This should not be deleted"), 11);
        table.delete_rows(vec![1, 2]).unwrap();
        let rows: Vec<Row> = table_iterator::RowsIterator::from_table(&table)
            .unwrap()
            .collect();
        assert!(table.drop().is_ok());
        assert_eq!(rows.len(), 2);
        assert_eq!(rows.get(0).unwrap(), &row);
        assert_eq!(rows.get(1).unwrap(), &row3);
    }

    pub fn insert_data(name: &str, indexed: bool) -> (Table, Row) {
        let table = create_table(name, indexed);
        table.create().unwrap();
        let row = insert_row(&table, String::from("Best SQL Server"), 8);
        (table, row)
    }

    pub fn insert_row(table: &Table, string: String, int: i32) -> Row {
        let string_value = string;
        let row = Row {
            values: vec![Data::STRING(string_value.clone()), Data::INT(int)],
        };
        assert!(table.insert_row(&row).is_ok());
        row
    }
}
