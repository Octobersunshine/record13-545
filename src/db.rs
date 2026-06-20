use sqlx::SqlitePool;

pub async fn init_db(database_url: &str) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect(database_url).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pets (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            species TEXT NOT NULL,
            breed TEXT,
            age INTEGER,
            owner_name TEXT NOT NULL,
            owner_phone TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS medical_records (
            id TEXT PRIMARY KEY,
            pet_id TEXT NOT NULL,
            visit_date TEXT NOT NULL,
            diagnosis TEXT NOT NULL,
            treatment TEXT NOT NULL,
            prescription TEXT,
            notes TEXT,
            veterinarian TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            deleted_at TEXT,
            FOREIGN KEY (pet_id) REFERENCES pets(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "ALTER TABLE medical_records ADD COLUMN deleted_at TEXT",
    )
    .execute(&pool)
    .await
    .ok();

    sqlx::query(
        "ALTER TABLE pets ADD COLUMN deleted_at TEXT",
    )
    .execute(&pool)
    .await
    .ok();

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_medical_records_pet_id ON medical_records(pet_id)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_medical_records_visit_date ON medical_records(visit_date DESC)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_medical_records_deleted_at ON medical_records(deleted_at)
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    Ok(pool)
}
