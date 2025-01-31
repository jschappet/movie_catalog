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
    sqlx::query(
        r#"
        CREATE VIRTUAL TABLE IF NOT EXISTS movies_fts USING fts5(title, description, content='movies', content_rowid='id');
        -- Automatically insert new movies into the FTS table
        CREATE TRIGGER movies_ai AFTER INSERT ON movies
        BEGIN
            INSERT INTO movies_fts(rowid, title, description) 
            VALUES (new.id, new.title, new.description);
        END;

        -- Automatically update the FTS table when a movie is updated
        CREATE TRIGGER movies_au AFTER UPDATE ON movies
        BEGIN
            DELETE FROM movies_fts WHERE rowid = old.id;
            INSERT INTO movies_fts(rowid, title, description) 
            VALUES (new.id, new.title, new.description);
        END;

        -- Automatically delete from the FTS table when a movie is removed
        CREATE TRIGGER movies_ad AFTER DELETE ON movies
        BEGIN
            DELETE FROM movies_fts WHERE rowid = old.id;
        END;

        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");
    pool
}
