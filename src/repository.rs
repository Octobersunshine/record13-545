use crate::models::{
    CreateMedicalRecordRequest, CreatePetRequest, DeleteResponse, MedicalRecord,
    PaginatedResponse, Pet, PurgeResponse, SanitizedMedicalRecord,
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
            INSERT INTO pets (id, name, species, breed, age, owner_name, owner_phone, created_at, updated_at, deleted_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
            RETURNING id, name, species, breed, age, owner_name, owner_phone, created_at, updated_at, deleted_at
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
            SELECT id, name, species, breed, age, owner_name, owner_phone, created_at, updated_at, deleted_at
            FROM pets
            WHERE id = ? AND deleted_at IS NULL
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
                (id, pet_id, visit_date, diagnosis, treatment, prescription, notes, veterinarian, created_at, updated_at, deleted_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
            RETURNING id, pet_id, visit_date, diagnosis, treatment, prescription, notes, veterinarian, created_at, updated_at, deleted_at
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
        include_deleted: bool,
    ) -> Result<Option<MedicalRecord>, sqlx::Error> {
        let where_clause = if include_deleted {
            "WHERE id = ?"
        } else {
            "WHERE id = ? AND deleted_at IS NULL"
        };

        let sql = format!(
            r#"
            SELECT id, pet_id, visit_date, diagnosis, treatment, prescription, notes, veterinarian, created_at, updated_at, deleted_at
            FROM medical_records
            {}
            "#,
            where_clause
        );

        let record = sqlx::query_as::<_, MedicalRecord>(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(record)
    }

    pub async fn soft_delete_medical_record(
        &self,
        id: Uuid,
    ) -> Result<Option<DeleteResponse>, sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            UPDATE medical_records
            SET deleted_at = ?, updated_at = ?
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(now)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            Ok(Some(DeleteResponse {
                success: true,
                id,
                deleted_at: now,
                message: "病历已软删除，敏感数据已脱敏".to_string(),
            }))
        } else {
            let exists = sqlx::query(
                r#"
                SELECT 1 FROM medical_records WHERE id = ?
                "#,
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

            if exists.is_some() {
                Ok(Some(DeleteResponse {
                    success: true,
                    id,
                    deleted_at: now,
                    message: "病历已处于删除状态".to_string(),
                }))
            } else {
                Ok(None)
            }
        }
    }

    pub async fn purge_medical_record(&self, id: Uuid) -> Result<PurgeResponse, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM medical_records WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            Ok(PurgeResponse {
                success: true,
                message: "病历已彻底物理删除".to_string(),
            })
        } else {
            Ok(PurgeResponse {
                success: false,
                message: "病历不存在".to_string(),
            })
        }
    }

    pub async fn list_medical_records_by_pet_id(
        &self,
        pet_id: Uuid,
        page: u32,
        page_size: u32,
        include_deleted: bool,
    ) -> Result<PaginatedResponse<SanitizedMedicalRecord>, sqlx::Error> {
        let offset = (page.saturating_sub(1)) as i64 * page_size as i64;
        let page_size_i64 = page_size as i64;

        let where_clause = if include_deleted {
            "WHERE pet_id = ?"
        } else {
            "WHERE pet_id = ? AND deleted_at IS NULL"
        };

        let items_sql = format!(
            r#"
            SELECT id, pet_id, visit_date, diagnosis, treatment, prescription, notes, veterinarian, created_at, updated_at, deleted_at
            FROM medical_records
            {}
            ORDER BY visit_date DESC, created_at DESC
            LIMIT ? OFFSET ?
            "#,
            where_clause
        );

        let items = sqlx::query_as::<_, MedicalRecord>(&items_sql)
            .bind(pet_id)
            .bind(page_size_i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let sanitized_items: Vec<SanitizedMedicalRecord> = items
            .into_iter()
            .map(|item| {
                if item.is_deleted() {
                    item.sanitize()
                } else {
                    SanitizedMedicalRecord {
                        id: item.id,
                        pet_id: item.pet_id,
                        visit_date: item.visit_date,
                        diagnosis: item.diagnosis,
                        treatment: item.treatment,
                        prescription: item.prescription,
                        notes: item.notes,
                        veterinarian: item.veterinarian,
                        created_at: item.created_at,
                        updated_at: item.updated_at,
                        deleted_at: item.deleted_at,
                        is_sanitized: false,
                    }
                }
            })
            .collect();

        let count_sql = format!(
            r#"
            SELECT COUNT(*) as count
            FROM medical_records
            {}
            "#,
            where_clause
        );

        let total_row = sqlx::query(&count_sql)
            .bind(pet_id)
            .fetch_one(&self.pool)
            .await?;

        let total: i64 = total_row.try_get("count")?;
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

        Ok(PaginatedResponse {
            items: sanitized_items,
            total,
            page,
            page_size,
            total_pages,
        })
    }
}
