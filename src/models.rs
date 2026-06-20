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
    pub deleted_at: Option<DateTime<Utc>>,
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
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SanitizedMedicalRecord {
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
    pub deleted_at: Option<DateTime<Utc>>,
    pub is_sanitized: bool,
}

impl MedicalRecord {
    pub fn sanitize(self) -> SanitizedMedicalRecord {
        let mask = |s: &str| -> String {
            if s.is_empty() {
                s.to_string()
            } else if s.len() <= 2 {
                "*".repeat(s.len())
            } else {
                let chars: Vec<char> = s.chars().collect();
                let first = chars[0];
                let last = chars[chars.len() - 1];
                let middle = "*".repeat(chars.len().saturating_sub(2));
                format!("{}{}{}", first, middle, last)
            }
        };

        let mask_middle = |s: &str, keep_start: usize, keep_end: usize| -> String {
            if s.len() <= keep_start + keep_end {
                "*".repeat(s.len())
            } else {
                let chars: Vec<char> = s.chars().collect();
                let start: String = chars.iter().take(keep_start).collect();
                let end: String = chars.iter().rev().take(keep_end).collect::<String>().chars().rev().collect();
                let middle = "*".repeat(s.len() - keep_start - keep_end);
                format!("{}{}{}", start, middle, end)
            }
        };

        SanitizedMedicalRecord {
            id: self.id,
            pet_id: self.pet_id,
            visit_date: self.visit_date,
            diagnosis: mask(&self.diagnosis),
            treatment: mask(&self.treatment),
            prescription: self.prescription.as_deref().map(|p| mask_middle(p, 1, 1)),
            notes: self.notes.as_deref().map(|n| mask(n)),
            veterinarian: mask_middle(&self.veterinarian, 1, 1),
            created_at: self.created_at,
            updated_at: self.updated_at,
            deleted_at: self.deleted_at,
            is_sanitized: true,
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

impl From<MedicalRecord> for SanitizedMedicalRecord {
    fn from(record: MedicalRecord) -> Self {
        record.sanitize()
    }
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
    #[serde(default)]
    pub include_deleted: bool,
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

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub id: Uuid,
    pub deleted_at: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PurgeResponse {
    pub success: bool,
    pub message: String,
}
