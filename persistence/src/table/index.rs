use common::models::db::{Data, Column};

use crate::table::errors::PersistenceErrors;
use std::{fs::write, collections::HashMap};

use super::{column::PersistenceColumn, row::PersistenceData};

#[derive(PartialEq, Debug)]
pub struct IndexRow {
    pub hash: u64,
    pub values: Vec<(Data, u64)>,
}

impl IndexRow {
    pub(crate) fn to_bytes(&self, column: &Column) -> Vec<u8> {
        let mut length: u64 = 0;
        let mut bytes = vec![self.hash.to_be_bytes().to_vec()];
        for (data, row_number) in &self.values {
            let column_size = column.size();
            bytes.push(data.to_bytes(column_size, &column.data_type));
            bytes.push(row_number.to_be_bytes().to_vec());
            length += column_size as u64 + 8;
        }
        vec![length.to_be_bytes().to_vec(), bytes.concat()].concat()
    }

    pub(crate) fn parse_u64(bytes: &[u8], cursor: usize) -> u64 {
        let length_bytes = bytes[cursor..cursor + 8].to_owned();
        u64::from_be_bytes([
            length_bytes[0],
            length_bytes[1],
            length_bytes[2],
            length_bytes[3],
            length_bytes[4],
            length_bytes[5],
            length_bytes[6],
            length_bytes[7],
        ])
    }

    pub(crate) fn from_bytes(bytes: Vec<u8>, column: &Column) -> IndexRow {
        let mut values_length = Self::parse_u64(&bytes, 0);
        let mut cursor: usize = 8;
        let hash = Self::parse_u64(&bytes, cursor);
        cursor += 8;
        let column_size = column.size();
        let mut values: Vec<(Data, u64)> = vec![];
        while values_length != 0 {
            let data_bytes = bytes[cursor..cursor + column_size].to_owned();
            let data = Data::from_bytes(data_bytes, column);
            cursor += column_size;
            let row_number = Self::parse_u64(&bytes, cursor);
            values.push((data, row_number));
            cursor += 8;
            values_length -= column_size as u64 + 8;
        }
        IndexRow { hash, values }
    }
}

#[derive(PartialEq, Debug)]
pub struct Index {
    pub rows: HashMap<u64, IndexRow>,
}

impl Index {
    fn to_bytes(&self, column: &Column) -> Vec<u8> {
        let mut bytes = vec![];
        for row in &self.rows {
            bytes.push(row.1.to_bytes(column));
        }
        bytes.concat()
    }

    fn from_bytes(bytes: Vec<u8>, column: &Column) -> Self {
        let mut rows = HashMap::new();
        let mut cursor: usize = 0;
        while cursor < bytes.len() {
            let length = IndexRow::parse_u64(&bytes, cursor) as usize + 16;
            let bytes = bytes[cursor..cursor + length].to_owned();
            let index_row = IndexRow::from_bytes(bytes, column);
            rows.insert(index_row.hash, index_row);
            cursor += length;
        }
        Index { rows }
    }

    pub(crate) fn write_index_to_file(
        &self,
        file_name: String,
        column: &Column,
    ) -> Result<(), PersistenceErrors> {
        write(file_name, self.to_bytes(column)).map_err(PersistenceErrors::IndexRefresh)?;
        Ok(())
    }

    pub(crate) fn load(file_name: String, column: &Column) -> Result<Self, PersistenceErrors> {
        let bytes = std::fs::read(file_name).map_err(PersistenceErrors::IndexLoading)?;
        Result::Ok(Index::from_bytes(bytes, column))
    }
}

#[cfg(test)]
pub mod tests {
    use std::fs::remove_file;
    use common::models::db::DataType;

    use super::*;

    #[test]
    fn index_row_int_to_and_from_bytes() {
        let int_column = Column {
            data_type: DataType::INT,
            is_indexed: true,
            name: String::from("id"),
        };
        let value1 = (Data::INT(1), 10);
        let value2 = (Data::INT(8), 5);
        let value3 = (Data::INT(5), 16);
        let index_row = IndexRow {
            hash: 123,
            values: vec![value1, value2, value3],
        };
        assert_eq!(
            index_row,
            IndexRow::from_bytes(index_row.to_bytes(&int_column), &int_column)
        );
    }

    #[test]
    fn index_row_string_to_and_from_bytes() {
        let string_column = Column {
            data_type: DataType::STRING { size: 255 },
            is_indexed: true,
            name: String::from("name"),
        };
        let value1 = (Data::STRING(String::from("Rust")), 10);
        let value2 = (Data::STRING(String::from("is")), 5);
        let value3 = (Data::STRING(String::from("great")), 16);
        let index_row = IndexRow {
            hash: 123,
            values: vec![value1, value2, value3],
        };
        assert_eq!(
            index_row,
            IndexRow::from_bytes(index_row.to_bytes(&string_column), &string_column)
        );
    }

    #[test]
    fn index_to_and_from_bytes() {
        let string_column = Column {
            data_type: DataType::STRING { size: 255 },
            is_indexed: true,
            name: String::from("name"),
        };
        let index_row1 = IndexRow {
            hash: 123,
            values: vec![
                (Data::STRING(String::from("Rust")), 10),
                (Data::STRING(String::from("is")), 5),
            ],
        };
        let index_row2 = IndexRow {
            hash: 566,
            values: vec![
                (Data::STRING(String::from("just")), 1),
                (Data::STRING(String::from("so")), 12),
            ],
        };
        let index_row3 = IndexRow {
            hash: 99,
            values: vec![
                (Data::STRING(String::from("great")), 99),
                (Data::STRING(String::from("man")), 69),
            ],
        };
        let index = Index {
            rows: HashMap::from([
                (index_row1.hash, index_row1),
                (index_row2.hash, index_row2),
                (index_row3.hash, index_row3),
            ]),
        };
        assert_eq!(
            index,
            Index::from_bytes(index.to_bytes(&string_column), &string_column)
        );
    }

    #[test]
    fn index_creation_and_loading() {
        let string_column = Column {
            data_type: DataType::STRING { size: 255 },
            is_indexed: true,
            name: String::from("name"),
        };
        let index_row1 = IndexRow {
            hash: 123,
            values: vec![
                (Data::STRING(String::from("Rust")), 10),
                (Data::STRING(String::from("is")), 5),
            ],
        };
        let index_row2 = IndexRow {
            hash: 566,
            values: vec![
                (Data::STRING(String::from("just")), 1),
                (Data::STRING(String::from("so")), 12),
            ],
        };
        let index_row3 = IndexRow {
            hash: 99,
            values: vec![
                (Data::STRING(String::from("great")), 99),
                (Data::STRING(String::from("man")), 69),
            ],
        };
        let index = Index {
            rows: HashMap::from([
                (index_row1.hash, index_row1),
                (index_row2.hash, index_row2),
                (index_row3.hash, index_row3),
            ]),
        };
        let file_name = String::from("index1");
        assert!(index
            .write_index_to_file(file_name.clone(), &string_column)
            .is_ok());
        let loaded_index = Index::load(file_name.clone(), &string_column).unwrap();
        assert_eq!(index, loaded_index);
        remove_file(file_name).unwrap();
    }
}
