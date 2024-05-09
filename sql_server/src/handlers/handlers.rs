use actix_web::{web, post};
use common::models::webserver_models::{QueryRequestData, QueryStatus, QueryResponseData};
use transaction_control::{process_query};
use std::{time::Instant};

use crate::models::AppState;


#[post("/query")]
pub async fn query(
    req: web::Json<QueryRequestData>,
    data: web::Data<AppState>
) -> web::Json<QueryResponseData> {
    let now = Instant::now();
    let result = process_query(&req.query, data.acid_sync.clone());

    match result {
        Ok(data) => web::Json(QueryResponseData {
            status: QueryStatus::Ok,
            data: data.data,
            message: data.message,
            duration: format!("{:.2} ms", (now.elapsed().as_nanos() as f32 / 1_000_000.0))
        }),
        Err(e) => web::Json(QueryResponseData {
            status: QueryStatus::Err,
            data: None,
            message: Some(format!("DB Error: {}", e)),
            duration: format!("{:.2} ms", (now.elapsed().as_nanos() as f32 / 1_000_000.0))
        })
    }
}