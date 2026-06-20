use crate::handlers::{
    create_medical_record, create_pet, get_medical_record, get_pet,
    health_check, list_medical_records_by_pet,
};
use crate::repository::Repository;
use axum::routing::{get, post};
use axum::Router;

pub fn create_router(repo: Repository) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/pets", post(create_pet))
        .route("/pets/:id", get(get_pet))
        .route("/medical-records", post(create_medical_record))
        .route("/medical-records/:id", get(get_medical_record))
        .route(
            "/pets/:pet_id/medical-records",
            get(list_medical_records_by_pet),
        )
        .with_state(repo)
}
