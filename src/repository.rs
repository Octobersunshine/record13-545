use crate::models::{
    CreateMedicalRecordRequest, CreatePetRequest, MedicalRecord, PaginatedResponse, Pet,
};
use chrono::Utc;
use sqlx::{SqlitePool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_pet(&self, req: CreatePetRequest) -> Result<Pet, sqlx::Error> {
        let now = Utc::now();
        let id = Uuid::new_v4();

        let pet = sqlx::query_as::<_, Pet>(
            r#"
            INSERT INTO pets (id, name, species, breed, age, owner_name, owner_phone, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, name, species, breed, age, owner_name, owner_phone, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.species)
        .bind(req.breed.as_deref())
        .bind(req.age)
        .bind(&req.owner_name)
        .bind(&req.owner_phone)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(pet)
    }

    pub async fn get_pet_by_id(&self, id: Uuid) -> Result<Option<Pet>, sqlx::Error> {
        let pet = sqlx::query_as::<_, Pet>(
            r#"
            SELECT id, name, species, breed, age, owner_name, owner_phone, created_at, updated_at
            FROM pets
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(pet)
    }

    pub async fn create_medical_record(
        &self,
        req: CreateMedicalRecordRequest,
    ) -> Result<MedicalRecord, sqlx::Error> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let visit_date = req.visit_date.unwrap_or(now);

        let record = sqlx::query_as::<_, MedicalRecord>(
            r#"
            INSERT INTO medical_records 
                (id, pet_id, visit_date, diagnosis, treatment, prescription, notes, veterinarian, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, pet_id, visit_date, diagnosis, treatment, prescription, notes, veterinarian, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(req.pet_id)
        .bind(visit_date)
        .bind(&req.diagnosis)
        .bind(&req.treatment)
        .bind(req.prescription.as_deref())
        .bind(req.notes.as_deref())
        .bind(&req.veterinarian)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_medical_record_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<MedicalRecord>, sqlx::Error> {
        let record = sqlx::query_as::<_, MedicalRecord>(
            r#"
            SELECT id, pet_id, visit_date, diagnosis, treatment, prescription, notes, veterinarian, created_at, updated_at
            FROM medical_records
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn list_medical_records_by_pet_id(
        &self,
        pet_id: Uuid,
        page: u32,
        page_size: u32,
    ) -> Result<PaginatedResponse<MedicalRecord>, sqlx::Error> {
        let offset = (page.saturating_sub(1)) as i64 * page_size as i64;
        let page_size_i64 = page_size as i64;

        let items = sqlx::query_as::<_, MedicalRecord>(
            r#"
            SELECT id, pet_id, visit_date, diagnosis, treatment, prescription, notes, veterinarian, created_at, updated_at
            FROM medical_records
            WHERE pet_id = ?
            ORDER BY visit_date DESC, created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(pet_id)
        .bind(page_size_i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total_row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM medical_records
            WHERE pet_id = ?
            "#,
        )
        .bind(pet_id)
        .fetch_one(&self.pool)
        .await?;

        let total: i64 = total_row.try_get("count")?;
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

        Ok(PaginatedResponse {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }
}
