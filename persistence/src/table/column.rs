use common::models::db::{Column, DataType};

pub trait PersistenceColumn {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: Vec<u8>) -> Column;
    fn size(&self) -> usize;
}

impl PersistenceColumn for Column {
    fn to_bytes(&self) -> Vec<u8> {
        return [
            (self.name.len() as u32).to_be_bytes().to_vec(),
            self.name.as_bytes().to_vec(),
            self.data_type.to_bytes().to_vec(),
            vec![self.is_indexed as u8],
        ]
        .concat();
    }

    fn from_bytes(bytes: Vec<u8>) -> Column {
        let name_size = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let name = match String::from_utf8(bytes[4usize..4usize + (name_size as usize)].to_owned())
        {
            Ok(string) => string,
            Err(_) => {
                panic!()
            }
        };
        let data_type_beginning = 4usize + name_size as usize;

        let is_indexed_beginning = data_type_beginning + 8;
        let data_type =
            DataType::from_bytes(bytes[data_type_beginning..is_indexed_beginning].to_owned());
        let is_indexed = (bytes[is_indexed_beginning] & 1) == 1;

        Column {
            name,
            data_type,
            is_indexed,
        }
    }

    fn size(&self) -> usize {
        match self.data_type {
            DataType::INT => 8,
            DataType::STRING { size } => size as usize,
            DataType::BOOLEAN => 8,
            DataType::FLOAT => 8,
        }
    }
}

pub trait PersistenceDataType {
    fn to_bytes(self) -> Vec<u8>;
    fn from_bytes(bytes: Vec<u8>) -> DataType;
}

impl PersistenceDataType for DataType {
    fn to_bytes(self) -> Vec<u8> {
        match self {
            DataType::INT => [0, 0, 0, 0, 0, 0, 0, 0].to_vec(),
            DataType::STRING { size } => [[1, 0, 0, 0], size.to_be_bytes()].concat(),
            DataType::BOOLEAN => [2, 0, 0, 0, 0, 0, 0, 0].to_vec(),
            DataType::FLOAT => [3, 0, 0, 0, 0, 0, 0, 0].to_vec(),
        }
    }

    fn from_bytes(bytes: Vec<u8>) -> DataType {
        match bytes[0] {
            0 => DataType::INT,
            1 => DataType::STRING {
                size: i32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            },
            2  => DataType::BOOLEAN,
            3  => DataType::FLOAT,
            _ => {
                panic!("Unknown DataType")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_to_and_from_bytes() {
        test_column_to_and_from_bytes(String::from("Name"), DataType::STRING { size: 255 }, true);
        test_column_to_and_from_bytes(String::from("Rust is just so cool"), DataType::INT, false);
        test_column_to_and_from_bytes(String::from("Rust is just so cool"), DataType::BOOLEAN, false);
        test_column_to_and_from_bytes(String::from("Rust is just so cool"), DataType::FLOAT, false);
    }

    fn test_column_to_and_from_bytes(name: String, data_type: DataType, is_indexed: bool) {
        let column = Column {
            name,
            data_type,
            is_indexed,
        };
        let column_from_bytes = Column::from_bytes(column.to_bytes());
        assert_eq!(column_from_bytes, column);
    }
}
