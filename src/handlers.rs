use crate::models::{
    CreateMedicalRecordRequest, CreatePetRequest, DeleteResponse, MedicalRecord,
    PaginatedResponse, PaginationQuery, Pet, PurgeResponse, SanitizedMedicalRecord,
};
use crate::pdf::generate_medical_record_pdf;
use crate::repository::Repository;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Json, Response};
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
) -> Result<impl IntoResponse, StatusCode> {
    let record = repo
        .get_medical_record_by_id(id, false)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match record {
        Some(record) => {
            if record.is_deleted() {
                let sanitized = record.sanitize();
                Ok((StatusCode::OK, Json(sanitized)).into_response())
            } else {
                Ok((StatusCode::OK, Json(record)).into_response())
            }
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn list_medical_records_by_pet(
    State(repo): State<Repository>,
    Path(pet_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<SanitizedMedicalRecord>>, StatusCode> {
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
        .list_medical_records_by_pet_id(pet_id, page, page_size, pagination.include_deleted)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(response))
}

pub async fn delete_medical_record(
    State(repo): State<Repository>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeleteResponse>, StatusCode> {
    let result = repo
        .soft_delete_medical_record(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Some(response) => Ok(Json(response)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn purge_medical_record(
    State(repo): State<Repository>,
    Path(id): Path<Uuid>,
) -> Result<Json<PurgeResponse>, StatusCode> {
    let result = repo
        .purge_medical_record(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.success {
        Ok(Json(result))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn download_medical_record_pdf(
    State(repo): State<Repository>,
    Path(id): Path<Uuid>,
) -> Result<Response, StatusCode> {
    let pdf_data = repo
        .get_medical_record_pdf_data(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match pdf_data {
        Some(data) => {
            let pdf_bytes = generate_medical_record_pdf(&data)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let filename = format!(
                "medical_record_{}_{}.pdf",
                data.pet.name,
                data.record.visit_date.format("%Y%m%d")
            );

            let body = Body::from(pdf_bytes);

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", filename),
                )
                .body(body)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
