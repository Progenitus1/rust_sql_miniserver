use serde::{Serialize, Deserialize};

use crate::models::db::{Column, Row};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableData {
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
}

#[derive(Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryStatus {
    Ok,
    #[default] Err,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct QueryResultData {
    pub data: Option<TableData>,
    pub message: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponseData {
    pub status: QueryStatus,
    pub data: Option<TableData>,
    pub message: Option<String>,
    pub duration: String
}


#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequestData {
    pub query: String,
}