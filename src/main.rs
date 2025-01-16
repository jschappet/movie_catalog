use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};

//use serde_json::Value;
use sqlx::{Encode, Row};

use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string};

use sqlx::query;
//use sqlx::{sqlite::SqlitePool, FromRow};

use std::env;
use std::sync::Arc;

use tower_http::services::{ServeDir, ServeFile};
mod db;
use db::{init_db, DbPool};

#[derive(Deserialize,Serialize)]
struct MovieList {
    titles: Vec<String>,
}



#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct MovieMetadata {
    title:  Option<String>,
    year:  Option<String>,
    rated:  Option<String>,
    released:  Option<String>,
    runtime:  Option<String>,
    genre:  Option<String>,
    director:  Option<String>,
    writer:  Option<String>,
    actors:  Option<String>,
    plot:  Option<String>,
    language:  Option<String>,
    country:  Option<String>,
    awards:  Option<String>,
    poster:  Option<String>,
    ratings: Option<Vec<Rating>>,
    metascore:  Option<String>,
    #[serde(rename = "imdbRating")]
    imdb_rating:  Option<String>,
    #[serde(rename = "imdbVotes")]
    imdb_votes:  Option<String>,
    #[serde(rename = "imdbID")]
    imdb_id:  Option<String>,
    #[serde(rename = "Type")]
    movie_type:  Option<String>, // Reserved keyword, escaped
    dvd: Option<String>,
    #[serde(rename = "BoxOffice")]
    box_office:  Option<String>,
    production:  Option<String>,
    website:  Option<String>,
    response: String,
    #[serde(rename = "Error")]
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Encode, sqlx::Type)]
#[serde(rename_all = "PascalCase")]
struct Rating {
    source: String,
    value: String,
}



#[tokio::main]
async fn main() {
    dotenv().ok();

    // Initialize database
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = init_db(&database_url).await;

    let app_state = Arc::new(AppState {
        db_pool,
        omdb_api_key: env::var("OMDB_API_KEY").expect("OMDB_API_KEY must be set"),
        client: Client::new(),
    });
    let serve_dir = ServeDir::new("static").not_found_service(ServeFile::new("static/index.html"));

    // Create Axum app
    let app = Router::new()
        .route("/upload", post(upload_movies))
        .route("/metadata", get(get_metadata))
        .nest_service("/", serve_dir.clone())
        .with_state(app_state);

    // Run server
    let addr = "127.0.0.1:3000";

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();
    //println!("listening on {}", listener.local_addr().unwrap());
    println!("Server running on http://{}", addr);

    axum::serve(listener, app).await.unwrap();

    /* 
    hyper::Server::bind(&addr)
    .serve(app.into_make_service())
        .await
        .unwrap();
    */
}

#[derive(Clone)]
struct AppState {
    db_pool: DbPool,
    omdb_api_key: String,
    client: Client,
}

async fn upload_movies(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MovieList>,
) -> String {
    let db = &state.db_pool;
    
    for title in payload.titles {
         
        let meta = fetch_metadata(&state, title.as_str()).await;
        match meta {
            Ok(meta) => {
                println!("{:?}", meta);

                let metadata: MovieMetadata = serde_json::from_str(&meta)
                    .expect("ERROR");

                let metadata_json = to_string(&meta).expect("Failed to serialize metadata");
                println!("{}", metadata_json);
                sqlx::query("INSERT INTO movies (title, metadata) VALUES (?, ?)")
                    .bind(title.clone())
                    .bind(json!(metadata))
                    .execute(db)
                    .await
                    .expect("Failed to insert movie");
            
                println!("Done Update: {}", title);
            }
            Err(e) => {
                sqlx::query("INSERT INTO movies (title) VALUES (?)")
                    .bind(title.clone())
                    .execute(db)
                    .await
                    .expect("Failed to insert movie");
                eprintln!("Failed to fetch metadata for {}: {:?}", title, e);
                () // Wrap it in a vector to return it as JSON
            }
        }   
    }
    "Movies uploaded successfully!".to_string()
}




async fn get_metadata(State(state): State<Arc<AppState>>) -> Json<Vec<MovieMetadata>> {
    let db = &state.db_pool;

    let rows = query("SELECT title, metadata FROM movies")
        .fetch_all(db)
        .await
        .expect("Failed to fetch movies");

    let mut metadata_list = Vec::new();
    for row in rows {
        //let title: String = row.get("title");
        let metadata: String = row.get("metadata");

        match serde_json::from_str::<MovieMetadata>(&metadata) {
            Ok(md) => {
                metadata_list.push(md); 
                    // Add the structured object to the list
                //println!("Parsed metadata: {:?}", md);
            }
            Err(e) => {
                eprintln!("Failed to parse metadata: {}", e);
            }
        }
        
        //if let Ok(metadata) = fetch_metadata(&state, &title).await {
            
        //}
    }

    Json(metadata_list)
}

async fn fetch_metadata(state: &AppState, title: &str) -> Result<String, reqwest::Error> {
    let url = format!(
        "http://www.omdbapi.com/?t={}&apikey={}",
        title, state.omdb_api_key
    );

    let res = state
        .client.get(&url)
        .send().await?
        .json::<serde_json::Value>().await?;
    Ok(res.to_string())
    /* Ok(MovieMetadata {
        title: res["Title"].as_str().unwrap_or_default().to_string(),
        year: res["Year"].as_str().unwrap_or_default().to_string(),
        genre: res["Genre"].as_str().unwrap_or_default().to_string(),
        director: res["Director"].as_str().unwrap_or_default().to_string(),
        plot: res["Plot"].as_str().unwrap_or_default().to_string(),
    })
    */
}
