use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Pet {
    pub id: Uuid,
    pub name: String,
    pub species: String,
    pub breed: Option<String>,
    pub age: Option<i32>,
    pub owner_name: String,
    pub owner_phone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MedicalRecord {
    pub id: Uuid,
    pub pet_id: Uuid,
    pub visit_date: DateTime<Utc>,
    pub diagnosis: String,
    pub treatment: String,
    pub prescription: Option<String>,
    pub notes: Option<String>,
    pub veterinarian: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMedicalRecordRequest {
    pub pet_id: Uuid,
    pub visit_date: Option<DateTime<Utc>>,
    pub diagnosis: String,
    pub treatment: String,
    pub prescription: Option<String>,
    pub notes: Option<String>,
    pub veterinarian: String,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    10
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePetRequest {
    pub name: String,
    pub species: String,
    pub breed: Option<String>,
    pub age: Option<i32>,
    pub owner_name: String,
    pub owner_phone: String,
}
