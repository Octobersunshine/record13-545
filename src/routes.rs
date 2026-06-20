use crate::handlers::{
    create_medical_record, create_pet, delete_medical_record, download_medical_record_pdf,
    get_medical_record, get_pet, health_check, list_medical_records_by_pet, purge_medical_record,
};
use crate::repository::Repository;
use axum::routing::{delete, get, post};
use axum::Router;

pub fn create_router(repo: Repository) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/pets", post(create_pet))
        .route("/pets/:id", get(get_pet))
        .route("/medical-records", post(create_medical_record))
        .route("/medical-records/:id", get(get_medical_record))
        .route("/medical-records/:id", delete(delete_medical_record))
        .route("/medical-records/:id/purge", delete(purge_medical_record))
        .route("/medical-records/:id/pdf", get(download_medical_record_pdf))
        .route(
            "/pets/:pet_id/medical-records",
            get(list_medical_records_by_pet),
        )
        .with_state(repo)
}
