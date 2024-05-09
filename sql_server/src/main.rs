use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};

mod handlers;
mod models;

use common::models::acid_sync::AcidSync;
use models::AppState;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let app_data = web::Data::new(AppState {
        acid_sync: AcidSync::default(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .wrap(Cors::default()
                .allow_any_origin()
                .allowed_methods(vec!["GET", "POST"])
                .allowed_header(http::header::CONTENT_TYPE)
                .allowed_header(http::header::ACCEPT)
                .max_age(3600))
            .service(handlers::query)
    }).bind("0.0.0.0:9000")?.workers(4).run().await?;

    Ok(())
}
