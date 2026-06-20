use crate::models::{
    CreateMedicalRecordRequest, CreatePetRequest, MedicalRecord, PaginatedResponse,
    PaginationQuery, Pet,
};
use crate::repository::Repository;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Json;
use uuid::Uuid;

pub async fn create_pet(
    State(repo): State<Repository>,
    Json(req): Json<CreatePetRequest>,
) -> Result<Json<Pet>, StatusCode> {
    let pet = repo
        .create_pet(req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(pet))
}

pub async fn get_pet(
    State(repo): State<Repository>,
    Path(id): Path<Uuid>,
) -> Result<Json<Pet>, StatusCode> {
    let pet = repo
        .get_pet_by_id(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match pet {
        Some(pet) => Ok(Json(pet)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_medical_record(
    State(repo): State<Repository>,
    Json(req): Json<CreateMedicalRecordRequest>,
) -> Result<Json<MedicalRecord>, StatusCode> {
    let pet_exists = repo
        .get_pet_by_id(req.pet_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if pet_exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let record = repo
        .create_medical_record(req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(record))
}

pub async fn get_medical_record(
    State(repo): State<Repository>,
    Path(id): Path<Uuid>,
) -> Result<Json<MedicalRecord>, StatusCode> {
    let record = repo
        .get_medical_record_by_id(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match record {
        Some(record) => Ok(Json(record)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn list_medical_records_by_pet(
    State(repo): State<Repository>,
    Path(pet_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<MedicalRecord>>, StatusCode> {
    let pet_exists = repo
        .get_pet_by_id(pet_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if pet_exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let page = pagination.page.max(1);
    let page_size = pagination.page_size.clamp(1, 100);

    let response = repo
        .list_medical_records_by_pet_id(pet_id, page, page_size)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(response))
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
