#[macro_use]
extern crate rocket;

use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::fs::{FileServer, NamedFile};
use rocket::State;
use std::path::Path;
use std::sync::Mutex;
use rocket_dyn_templates::{Template, context};
//use sqlx::types::Json;
use sqlx::query;

use dotenv::dotenv;
use std::env;
use sqlx::{Encode, Row};


mod db;
use db::{init_db, DbPool};

// Struct to represent a user
#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone here
#[serde(crate = "rocket::serde")]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Application state
#[derive(Clone)]
struct AppState {
    db_pool: DbPool,
    omdb_api_key: String,
    client: reqwest::Client,
}

/* // Handler to get a user by ID
#[get("/<id>")]
async fn get_user(id: u32, state: &State<AppState>) -> Option<Json<User>> {
    let users = state.users.lock().unwrap();
    if let Some(user) = users.iter().find(|user| user.id == id) {
        println!("Page Title: {}", state.title);
        return Some(Json(user.clone()));
    }
    None
} */

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


#[get("/movies")]
async fn movies(state: &State<AppState>) -> Template {

    let db = &state.db_pool;

    let rows = query("SELECT title, metadata FROM movies")
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
            Err(e) => {
                eprintln!("Failed to parse metadata: {}", e);
            }
        }
        
    }

    Template::render("movie_card", context! { movies })
}

/* // Handler to add a new user
#[post("/", data = "<new_user>")]
async fn add_user(new_user: Json<User>, state: &State<AppState>) -> &'static str {
    let mut users = state.users.lock().unwrap();
    users.push(new_user.into_inner());
    println!("Page Title: {}", state.title);
    "User added successfully!"
} */

// Fallback to serve the `index.html` file when no route matches
#[get("/")]
async fn serve_index(state: &State<AppState>) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/index.html")).await.ok()
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
        .mount("/", routes![movies])
        .mount("/foo", routes![serve_index])
        .mount("/", FileServer::from("static")) // Serve static files
        .attach(Template::fairing())

}
