use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use common::models::db::{Column, Data, DataType, Row};

use super::column::PersistenceColumn;

pub trait PersistenceRow {
    fn from_bytes(bytes: Vec<u8>, columns: &Vec<Column>) -> Row;
    fn to_bytes(&self, columns: &[Column]) -> Vec<u8>;
}

impl PersistenceRow for Row {
    fn from_bytes(bytes: Vec<u8>, columns: &Vec<Column>) -> Row {
        let mut values = vec![];
        let mut byte_counter = 0;
        for column in columns {
            let data_size = match column.data_type {
                DataType::STRING { size } => {
                    size as usize
                }
                _ => 8usize
            };
            values.push(Data::from_bytes(
                bytes[byte_counter..byte_counter + data_size].to_owned(),
                column
            ));
            byte_counter += data_size;
        }
        Row { values }
    }

    fn to_bytes(&self, columns: &[Column]) -> Vec<u8> {
        let mut byte_vectors: Vec<Vec<u8>> = vec![];
        for (index, column) in columns.iter().enumerate() {
            byte_vectors.push(
                self.values
                    .get(index)
                    .unwrap()
                    .to_bytes(column.size(), &column.data_type),
            );
        }
        byte_vectors.concat()
    }
}

pub trait PersistenceData {
    fn to_bytes(&self, max_size: usize, data_type: &DataType) -> Vec<u8>;
    fn int_from_bytes(bytes: Vec<u8>) -> Data;
    fn string_from_bytes(bytes: Vec<u8>) -> Data;
    fn boolean_from_bytes(bytes: Vec<u8>) -> Data;
    fn float_from_bytes(bytes: Vec<u8>) -> Data;
    fn from_bytes(bytes: Vec<u8>, column: &Column) -> Self;
    fn calculate_hash(&self) -> u64;
}

impl PersistenceData for Data {
    fn to_bytes(&self, max_size: usize, data_type: &DataType) -> Vec<u8> {
        return match &self {
            Data::INT(integer) => [0_i32.to_be_bytes(), integer.to_be_bytes()].concat(),
            Data::STRING(string) => {
                let mut string_bytes = string.as_bytes().to_vec();
                if string_bytes.len() > max_size {
                    panic!();
                }
                while string_bytes.len() < max_size {
                    string_bytes.push(0);
                }

                string_bytes
            }
            Data::NULL => match data_type {
                DataType::INT => [1, 0, 0, 0, 0, 0, 0, 0].to_vec(),
                DataType::STRING { size: _ } => [0, 0, 0, 0, 0, 0, 0, 0].to_vec(),
                DataType::BOOLEAN => [0, 0, 0, 0, 0, 0, 0, 0].to_vec(),
                DataType::FLOAT => [0, 0, 0, 0, 0, 0, 0, 0].to_vec(),
            },
            Data::BOOLEAN(bool) => {
                let bool_representation: u8 = if *bool { 1 } else { 0 };
                [1, 1, bool_representation, 0, 0, 0, 0, 0].to_vec()
            }
            Data::FLOAT(float) => [float.to_be_bytes()].concat(),
        };
    }

    fn int_from_bytes(bytes: Vec<u8>) -> Data {
        Data::INT(i32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]))
    }

    fn string_from_bytes(bytes: Vec<u8>) -> Data {
        let vec: Vec<u8> = bytes
            .iter()
            .take_while(|byte| **byte != 0)
            .copied()
            .collect();
        Data::STRING(match String::from_utf8(vec) {
            Ok(result) => result,
            Err(_) => panic!(),
        })
    }

    fn boolean_from_bytes(bytes: Vec<u8>) -> Data {
        Data::BOOLEAN(bytes[2] != 0)
    }

    fn float_from_bytes(bytes: Vec<u8>) -> Data {
        let bytes_array = bytes.try_into().unwrap_or_else(|bytes: Vec<u8>| {
            panic!("Expected a Vec of length {} but it was {}", 8, bytes.len())
        });
        Data::FLOAT(f64::from_be_bytes(bytes_array))
    }

    fn from_bytes(bytes: Vec<u8>, column: &Column) -> Self {
        match column.data_type {
            DataType::INT => {
                let null = [1, 0, 0, 0, 0, 0, 0, 0];
                if bytes.eq(&null) {
                    return Data::NULL;
                }
                Self::int_from_bytes(bytes)
            }
            DataType::STRING { size: _size } => {
                let null = [0, 0, 0, 0, 0, 0, 0, 0];
                if bytes.eq(&null) {
                    return Data::NULL;
                }
                Self::string_from_bytes(bytes)
            }
            DataType::BOOLEAN => {
                let null = [0, 0, 0, 0, 0, 0, 0, 0];
                if bytes.eq(&null) {
                    return Data::NULL;
                }
                Self::boolean_from_bytes(bytes)
            }
            DataType::FLOAT => {
                let null = [0, 0, 0, 0, 0, 0, 0, 0];
                if bytes.eq(&null) {
                    return Data::NULL;
                }
                Self::float_from_bytes(bytes)
            }
        }
    }

    fn calculate_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::table::column::PersistenceDataType;

    use super::*;

    #[test]
    fn one_into_bytes() {
        let one = Data::INT(1);
        let data_type = DataType::INT;
        assert_eq!(one.to_bytes(255, &data_type), [0, 0, 0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn one_from_bytes() {
        let one = Data::int_from_bytes([0, 0, 0, 0, 0, 0, 0, 1].to_vec());
        assert_eq!(Data::INT(1), one)
    }

    #[test]
    fn big_int_into_bytes() {
        let big_int = Data::INT(78456845);
        let data_type = DataType::INT;
        assert_eq!(
            big_int.to_bytes(255, &data_type),
            [0, 0, 0, 0, 4, 173, 40, 13]
        );
    }

    #[test]
    fn big_int_from_bytes() {
        let big_int = Data::int_from_bytes([0, 0, 0, 0, 4, 173, 40, 13].to_vec());
        assert_eq!(Data::INT(78456845), big_int);
    }

    #[test]
    fn string_into_bytes() {
        let hello_world = Data::STRING(String::from("Hello word"));
        let expected_result = [
            72, 101, 108, 108, 111, 32, 119, 111, 114, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let data_type = DataType::INT;
        assert_eq!(hello_world.to_bytes(20, &data_type), expected_result);
    }

    #[test]
    fn string_from_bytes() {
        let bytes = [
            72, 101, 108, 108, 111, 32, 119, 111, 114, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let hello_world = Data::string_from_bytes(bytes.to_vec());
        assert_eq!(Data::STRING(String::from("Hello word")), hello_world);
    }

    #[test]
    fn int_null_from_bytes() {
        let bytes = [1, 0, 0, 0, 0, 0, 0, 0];
        let column = Column {
            data_type: DataType::INT,
            is_indexed: false,
            name: String::from("abc"),
        };
        let null = Data::from_bytes(bytes.to_vec(), &column);
        assert_eq!(null, Data::NULL)
    }

    #[test]
    fn string_null_from_bytes() {
        let bytes = [0, 0, 0, 0, 0, 0, 0, 0];
        let column = Column {
            data_type: DataType::STRING { size: 256 },
            is_indexed: false,
            name: String::from("abc"),
        };
        let null = Data::from_bytes(bytes.to_vec(), &column);
        assert_eq!(null, Data::NULL)
    }

    #[test]
    fn null_to_bytes() {
        let bytes = [0, 0, 0, 0, 0, 0, 0, 0].to_vec();
        let data_type = DataType::STRING { size: 256 };
        assert_eq!(bytes, Data::NULL.to_bytes(256, &data_type));
        let bytes = [1, 0, 0, 0, 0, 0, 0, 0].to_vec();
        let data_type = DataType::INT;
        assert_eq!(bytes, Data::NULL.to_bytes(256, &data_type));
    }

    #[test]
    fn int_data_type_to_bytes() {
        let int_data_type = DataType::INT;
        assert_eq!(int_data_type.to_bytes(), [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn int_data_type_from_bytes() {
        let int_data_type = DataType::from_bytes([0, 0, 0, 0, 0, 0, 0, 0].to_vec());
        match int_data_type {
            DataType::INT => {
                assert_eq!(1, 1);
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn string_data_type_to_bytes() {
        let string_data_type = DataType::STRING { size: 255 };
        assert_eq!(string_data_type.to_bytes(), [1, 0, 0, 0, 0, 0, 0, 255]);
    }

    #[test]
    fn string_data_type_from_bytes() {
        let string_data_type = DataType::from_bytes([1, 0, 0, 0, 0, 0, 0, 255].to_vec());
        match string_data_type {
            DataType::STRING { size } => {
                assert_eq!(size, 255);
            }
            _ => {
                panic!()
            }
        }
    }

    #[test]
    fn row_hash_int() {
        let int = Data::INT(8);
        let mut hasher = DefaultHasher::new();
        int.hash(&mut hasher);
        let hash = hasher.finish();
        assert_eq!(hash, 17499417595158719687);
    }

    #[test]
    fn row_hash_string() {
        let int = Data::STRING(String::from("Rust is just so cool."));
        let mut hasher = DefaultHasher::new();
        int.hash(&mut hasher);
        let hash = hasher.finish();
        assert_eq!(hash, 15568559637832932389);
    }

    #[test]
    fn row_to_and_from_bytes() {
        let string_data_type = DataType::STRING { size: 500 };
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
        let bool_column = Column {
            name: String::from("Bool"),
            data_type: DataType::BOOLEAN,
            is_indexed: false,
        };
        let float_column = Column {
            name: String::from("Float"),
            data_type: DataType::FLOAT,
            is_indexed: false,
        };

        let columns = vec![column_name, column_id, bool_column, float_column];

        let int = Data::INT(8);
        let string_value = String::from("Best SQL Server");
        let string = Data::STRING(string_value.clone());
        let bool_data = Data::BOOLEAN(true);
        let float_data = Data::FLOAT(45.675f64);
        let row = Row {
            values: vec![string, int, bool_data, float_data],
        };
        let bytes = row.to_bytes(&columns);
        let loaded_row = Row::from_bytes(bytes, &columns);
        match loaded_row.values.get(0).unwrap() {
            Data::STRING(value) => {
                assert_eq!(&string_value, value);
            }
            _ => {
                panic!();
            }
        }
        match loaded_row.values.get(1).unwrap() {
            Data::INT(value) => {
                assert_eq!(&8, value);
            }
            _ => {
                panic!();
            }
        }
        match loaded_row.values.get(2).unwrap() {
            Data::BOOLEAN(value) => {
                assert_eq!(true, *value);
            }
            _ => panic!(),
        }
        match loaded_row.values.get(3).unwrap() {
            Data::FLOAT(value) => {
                assert_eq!(45.675f64, *value);
            }
            _ => panic!(),
        }
    }
}
