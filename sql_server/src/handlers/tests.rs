#[cfg(test)]
mod integration_tests {

    use actix_web::{App};
    use actix_web::http::header::ContentType;
    use actix_web::test::{TestRequest, init_service, call_service, read_body_json};
    use actix_web::web::{Data};
    use common::models::acid_sync::AcidSync;
    use common::models::webserver_models::{QueryRequestData, QueryStatus, QueryResponseData};
    use crate::handlers;
    use crate::models::AppState;

    fn setup_requst(payload: String) -> TestRequest {
        TestRequest::post()
            .insert_header(ContentType::json())
            .uri("/query")
            .set_json(QueryRequestData {
                query: payload
            })
    }

    #[actix_web::test]
    async fn simple_table_creation() {
        let app_data = Data::new(AppState { acid_sync: AcidSync::default() });
        let app = init_service(App::new().app_data(app_data.clone()).service(handlers::query)).await;
        
        let req_create = setup_requst("CREATE TABLE employees name varchar, age int".to_string());
        let resp_create = call_service(&app, req_create.to_request()).await;
        assert!(resp_create.status().is_success());
        
        let body_create: QueryResponseData = read_body_json(resp_create).await;
        assert_eq!(body_create.status, QueryStatus::Ok);
        assert_eq!(body_create.message, Some("Table employees created.".to_string()));

        let req_drop = setup_requst("DROP TABLE employees".to_string());
        let resp_drop = call_service(&app, req_drop.to_request()).await;
        assert!(resp_drop.status().is_success());
        
        let body_drop: QueryResponseData = read_body_json(resp_drop).await;
        assert_eq!(body_drop.status, QueryStatus::Ok);
        assert_eq!(body_drop.message, Some("Table employees dropped.".to_string()));
    }

    #[actix_web::test]
    async fn insert_data_in_table() {
        let app_data = Data::new(AppState { acid_sync: AcidSync::default() });
        let app = init_service(App::new().app_data(app_data.clone()).service(handlers::query)).await;
        
        // Crete table
        let req_create = setup_requst("CREATE TABLE people name varchar, age int".to_string());
        let resp_create = call_service(&app, req_create.to_request()).await;
        assert!(resp_create.status().is_success());
        
        let body_create: QueryResponseData = read_body_json(resp_create).await;
        assert_eq!(body_create.status, QueryStatus::Ok);

        // Insert data
        let req_insert_1 = setup_requst("INSERT INTO people VALUES 'John', 30".to_string());
        let resp_insert_1 = call_service(&app, req_insert_1.to_request()).await;
        assert!(resp_insert_1.status().is_success());

        let body_insert_1: QueryResponseData = read_body_json(resp_insert_1).await;
        assert_eq!(body_insert_1.status, QueryStatus::Ok);


        let req_insert_2 = setup_requst("INSERT INTO people VALUES 'Jane', 25".to_string());
        let resp_insert_2 = call_service(&app, req_insert_2.to_request()).await;
        assert!(resp_insert_2.status().is_success());

        let body_insert_2: QueryResponseData = read_body_json(resp_insert_2).await;
        assert_eq!(body_insert_2.status, QueryStatus::Ok);

        // Delete data
        let req_delete = setup_requst("DELETE FROM people WHERE age > 10".to_string());
        let resp_delete = call_service(&app, req_delete.to_request()).await;
        assert!(resp_delete.status().is_success());

        let body_delete: QueryResponseData = read_body_json(resp_delete).await;
        assert_eq!(body_delete.status, QueryStatus::Ok);

        // Drop table
        let req_drop = setup_requst("DROP TABLE people".to_string());
        let resp_drop = call_service(&app, req_drop.to_request()).await;
        assert!(resp_drop.status().is_success());
        
        let body_drop: QueryResponseData = read_body_json(resp_drop).await;
        assert_eq!(body_drop.status, QueryStatus::Ok);
    }

}