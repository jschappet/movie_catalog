#[macro_use]
extern crate rocket;

use regex::Regex;

// use rocket::serde::{Deserialize, Serialize};
use rocket::fs::{FileServer, NamedFile};
use rocket::State;
use std::path::Path;
//use std::sync::Mutex;
use rocket_dyn_templates::{Template, context};
//use sqlx::types::Json;
use sqlx::query;

use dotenv::dotenv;
use std::env;
use sqlx::Row;

mod movies;
use movies::MovieMetadata;


mod db;
use db::{init_db, DbPool};


// Application state
#[derive(Clone)]
struct AppState {
    db_pool: DbPool,
    omdb_api_key: String,
    client: reqwest::Client,
}


#[get("/movies")]
async fn get_movies(state: &State<AppState>) -> Template {

    let db = &state.db_pool;

    let rows = query("SELECT title, metadata FROM movies order by title")
        .fetch_all(db)
        .await
        .expect("Failed to fetch movies");

    let mut movies = Vec::new();
    for row in rows {
        //let title: String = row.get("title");
        let metadata: String = row.get("metadata");

        match serde_json::from_str::<MovieMetadata>(&metadata) {
            Ok(md) => {
                //println!("Parsed metadata: {:?}", md);

                movies.push(md); 
                    // Add the structured object to the list
            }
            Err(_) => {
                //eprintln!("Failed to parse metadata: {}", e);
            }
        }
    
    }

    Template::render("movie_card", context! { movies })
}


// Fallback to serve the `index.html` file when no route matches
#[get("/")]
async fn serve_index() -> Option<NamedFile> {
    NamedFile::open(Path::new("static/index.html")).await.ok()
}

// Fallback to serve the `index.html` file when no route matches
#[delete("/metadata")]
async fn delete_metadata(state: &State<AppState>) ->  &'static str {
    let db: &sqlx::Pool<sqlx::Sqlite> = &state.db_pool;

    sqlx::query("UPDATE movies SET metadata=''")
    .execute(db)
    .await
    .expect("Failed to delete metadata");
    "All Movie Metadata Deleted"
}

fn escape_plot(plot: Option<String>) -> Option<String> {
    plot.map(|p| p.replace("'", "&#39;"))
}

fn process_title_and_year(title_with_year: &str) -> (String, Option<String>) {
    // Regular expression to capture the title and year in parentheses
    let re = Regex::new(r"^(.*)\s\((\d{4})\)$").unwrap();

    if let Some(captures) = re.captures(title_with_year) {
        // Extract the title and year
        let title = captures.get(1).map_or("", |m| m.as_str()).to_string();
        let year = captures.get(2).map_or("", |m| m.as_str()).to_string();
        (title, Some(year))
    } else {
        // If no match, return the original string and None for the year
        (title_with_year.to_string(), None)
    }
}


async fn fetch_metadata(state: &AppState, title: &str) -> Result<String, reqwest::Error> {

    let (title_without_year, year) = process_title_and_year(title);

    let url = match year {
        Some(y) => format!(
            "http://www.omdbapi.com/?t={}&y={}&apikey={}",
            title_without_year.replace("&", "%26"), y,  state.omdb_api_key
        ),
        None => format!(
            "http://www.omdbapi.com/?t={}&apikey={}",
            title_without_year.replace("&", "%26"),  state.omdb_api_key
        ),
    };

    /* let url = format!(
        "http://www.omdbapi.com/?t={}&apikey={}",
        title_without_year.replace("&", "%26"), state.omdb_api_key
    ); */

    let res = state
        .client.get(&url)
        .send().await?
        .json::<serde_json::Value>().await?;
    Ok(res.to_string())
    
}

// Handler for file uploads
#[post("/upload", data = "<data>")]
async fn upload_movies(state: &State<AppState>, data: String) -> &'static str {
    let db: &sqlx::Pool<sqlx::Sqlite> = &state.db_pool;
    
    // Parse JSON data
    let parsed_data: serde_json::Value = match serde_json::from_str(&data) {
        Ok(json) => json,
        Err(_) => {
            return "Invalid JSON format";
        }
    };
    if let Some(titles) = parsed_data["titles"].as_array() {

        for title in titles {
            
            let meta = fetch_metadata(&state, 
                    title.as_str().unwrap_or_default()).await;
            match meta {
                Ok(meta) => {
                    println!("{:?}", meta);

                    let mut metadata: MovieMetadata = serde_json::from_str(&meta)
                        .expect("ERROR");

                    let metadata_json = serde_json::to_string(&meta).expect("Failed to serialize metadata");
                    println!("{}", metadata_json);
                    if metadata.response == "False" {
                        metadata.title = Some(String::from(title.as_str().unwrap_or(""))); 
                    } 
                    metadata.plot = escape_plot(metadata.plot);
                    metadata.title = escape_plot(metadata.title);
                    

                    sqlx::query("INSERT INTO movies (title, metadata) VALUES (?, ?)")
                    .bind(title.clone())
                    .bind(serde_json::json!(metadata))
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
    }
    "Movies uploaded successfully!"
}

// Main Rocket launch function
#[launch]
async fn rocket() -> _ {

    // Load environment variables
    dotenv().ok();

    // Initialize database (mocking with a simple string)
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = init_db(&database_url).await;

    let app_state = AppState {
        db_pool,
        omdb_api_key: env::var("OMDB_API_KEY").expect("OMDB_API_KEY must be set"),
        client: reqwest::Client::new(),
    };

    rocket::build()
        .manage(app_state) // Add shared state
        .mount("/", routes![get_movies, upload_movies, delete_metadata])
        .mount("/foo", routes![serve_index])
        .mount("/", FileServer::from("static")) // Serve static files
        .attach(Template::fairing())

}
