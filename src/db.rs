use sqlx::{sqlite::SqlitePool, Pool, Sqlite};

pub type DbPool = Pool<Sqlite>;

pub async fn init_db(database_url: &str) -> DbPool {
    let pool = SqlitePool::connect(database_url)
        .await.expect("Failed to connect to the database");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS movies (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL ,
            source TEXT,
            imdbid TEXT UNIQUE,
            year INTEGER,
            metadata JSON,
            UNIQUE(title, year)
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");
    pool
}
