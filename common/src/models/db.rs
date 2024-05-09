use std::{fmt, hash::Hash, hash::Hasher};

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
pub enum DataType {
    INT,
    STRING { size: i32 },
    BOOLEAN,
    FLOAT,
}

impl DataType {
    fn is_string(&self) -> bool {
        match &self {
            #[allow(unused_variables)]
            DataType::STRING { size } => true,
            _ => false,
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub is_indexed: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Data {
    INT(i32),
    STRING(String),
    NULL,
    BOOLEAN(bool),
    FLOAT(f64),
}

impl Data {
    pub fn is_valid_data_for_type(&self, data_type: &DataType) -> bool {
        match self {
            Data::INT(_) => data_type.eq(&DataType::INT),
            Data::STRING(_) => data_type.is_string(),
            Data::NULL => true,
            Data::BOOLEAN(_) => data_type.eq(&DataType::BOOLEAN),
            Data::FLOAT(_) => data_type.eq(&DataType::FLOAT),
        }
    }

    pub fn to_type(&self) -> String {
        match self {
            Data::INT(_) => String::from("INT"),
            Data::STRING(_) => String::from("STRING"),
            Data::NULL => String::from("UNKNOWN since value was null"),
            Data::BOOLEAN(_) => String::from("BOOLEAN"),
            Data::FLOAT(_) => String::from("FLOAT"),
        }
    }
}

impl Hash for Data {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Data::INT(val) => val.hash(state),
            Data::STRING(string) => string.hash(state),
            Data::NULL => state.write_u8(0),
            Data::BOOLEAN(bool) => bool.hash(state),
            Data::FLOAT(val) => {
                let integer_part = *val as i64;
                let fractional_part = get_frac(*val);
                integer_part.hash(state);
                fractional_part.hash(state);
            }
        }
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {

        match (self, other) {
            (Data::INT(a), Data::INT(b)) => a == b,
            (Data::STRING(a), Data::STRING(b)) => a == b,
            (Data::BOOLEAN(a), Data::BOOLEAN(b)) => a == b,
            (Data::FLOAT(a), Data::FLOAT(b)) => a == b,
            (Data::NULL, Data::NULL) => true,
            _ => false,
        }
    }
}

fn get_frac(f: f64) -> u64 {
    let eps = 1e-4;
    let mut f = f.abs().fract();
    if f == 0.0 {
        return 0;
    }

    while (f.round() - f).abs() <= eps {
        f *= 10.0;
    }

    while (f.round() - f).abs() > eps {
        f *= 10.0;
    }

    f.round() as u64
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub values: Vec<Data>,
}

impl serde::ser::Serialize for DataType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            DataType::INT => serializer.serialize_str("INT"),
            DataType::STRING { size: _ } => serializer.serialize_str("STRING"),
            DataType::BOOLEAN => serializer.serialize_str("BOOLEAN"),
            DataType::FLOAT => serializer.serialize_str("FLOAT"),
        }
    }
}

impl serde::ser::Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Data::INT(val) => serializer.serialize_newtype_variant("Data", 0, "INT", &val),
            Data::STRING(val) => serializer.serialize_newtype_variant("Data", 1, "STRING", &val),
            Data::NULL => serializer.serialize_newtype_variant("Data", 2, "NULL", &"NULL"),
            Data::BOOLEAN(val) => {
                serializer.serialize_newtype_variant("Data", 3, "BOOLEAN", &val.to_string())
            }
            Data::FLOAT(val) => serializer.serialize_newtype_variant("Data", 4, "FLOAT", &val),
        }
    }
}
