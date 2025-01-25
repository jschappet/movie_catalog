#[macro_use]
extern crate rocket;

use regex::Regex;

// use rocket::serde::{Deserialize, Serialize};
use rocket::fs::{FileServer, NamedFile, TempFile};
use rocket::response::content::RawHtml;
use rocket::form::Form;


use rocket::time::format_description::parse;
use rocket::State;
use std::path::Path;
//use std::sync::Mutex;
use rocket_dyn_templates::{Template, context};
//use sqlx::types::Json;
use sqlx::query;
use rocket::tokio::fs;


use dotenv::dotenv;
use std::env;
use sqlx::Row;

mod movies;
use movies::MovieMetadata;


mod db;
use db::{init_db, DbPool};

#[derive(FromForm)]
struct FileUpload<'r> {
    file: TempFile<'r>,
}

#[post("/saved", data = "<file_upload>")]
async fn saved_file(mut file_upload: Form<FileUpload<'_>>) -> &'static str {
    // Define the upload directory
    let upload_dir = "./uploads";

    // Ensure the directory exists
    fs::create_dir_all(upload_dir)
        .await
        .expect("Failed to create upload directory");

    // Save the file to the upload directory
    if let Some(filename) = file_upload.file.name() {
        let save_path = format!("{}/{}", upload_dir, filename);
        if let Err(e) = file_upload.file.persist_to(save_path).await {
            eprintln!("Failed to save file: {}", e);
            return "Failed to save file.";
        }
        "File uploaded successfully!"
    } else {
        "No file was uploaded."
    }
}

#[get("/save")]
async fn save_file() -> RawHtml<&'static str> {
    RawHtml(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Upload File</title>
        </head>
        <body>
            <h1>File Upload</h1>
            <form action="/saved" method="post" enctype="multipart/form-data">
                <input type="file" name="file" required />
                <button type="submit">Upload</button>
            </form>
        </body>
        </html>
        "#,
    )
}

// Application state
#[derive(Clone)]
struct AppState {
    db_pool: DbPool,
    omdb_api_key: String,
    client: reqwest::Client,
}

fn get_first_letter(input: Option<String>) -> String {
    input.unwrap().chars().next()
        .map(|c| c.to_string()).unwrap_or("0".to_string())
}

#[get("/movies")]
async fn get_movies(state: &State<AppState>) -> Template {

    let db = &state.db_pool;

    let rows = query(r#"
        SELECT imdbid, title, source, metadata 
        FROM movies 
        ORDER by json_extract(metadata , '$.Title') 
    "#)
        .fetch_all(db)
        .await
        .expect("Failed to fetch movies");

    let mut movies = Vec::new();
    for row in rows {
        //let title: String = row.get("title");
        let metadata: String = row.get("metadata");

        //let source: String = row.get("source");

        match serde_json::from_str::<MovieMetadata>(&metadata) {
            Ok(mut md) => {
                //println!("Source: {:?}", source);
                //md.source = Some(source);
                md.first_letter = Some(get_first_letter(md.title.clone()));
                    
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



#[get("/refreshdata/<id>")]
async fn refresh_metadata(state: &State<AppState>, id: &str) ->  String  {
    let db = &state.db_pool;

    let rows = query("SELECT id, source, title, metadata FROM movies  WHERE imdbid = ?")
        .bind(id)
        .fetch_all(db)
        .await
        .expect("Failed to fetch movies");
        let mut movie = String::new();
        for row in rows {
            let title: String = row.get("title");
            let source: String = row.get("source");

            update_metadata(state, title.clone(), source).await;
            movie = String::from(title);
        }
        format!("Updated Movie: {}", movie)
}


#[get("/movie/<id>")]
async fn get_movie(state: &State<AppState>, id: &str) -> Template {
    let db = &state.db_pool;
    println!("ID: {}", id);

    // Query the database
    let rows = query("SELECT id, year, source, title, metadata FROM movies WHERE imdbid = ?")
        .bind(id)
        .fetch_all(db)
        .await;

    // Handle query errors gracefully
    let rows = match rows {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Database query failed: {:?}", e);
            return Template::render("error", context! { id });
        }
    };

    // Process the results
    for row in rows {
        let metadata: String = row.get("metadata");
        let year: i32 = row.get("year");
        println!("Year: {}", year);

        // Attempt to deserialize metadata
        match serde_json::from_str::<MovieMetadata>(&metadata) {
            Ok(md) => {
                return Template::render("movie_view", context! { movie: md });
            }
            Err(_) => {
                eprintln!("Failed to parse metadata: {}", metadata);
                return  Template::render("error", context! { id });

            }
        }
    }

    // If no rows were found, render an error template
    Template::render("error", context! { id })
}



#[get("/missingdata")]
async fn get_missing_data(state: &State<AppState>) ->  String  {

    let db = &state.db_pool;

    let rows = query("SELECT id, title, source FROM movies  WHERE json_extract(metadata, '$.Response') ='False' order by title")
        .fetch_all(db)
        .await
        .expect("Failed to fetch movies");

    let mut movies = Vec::new();
    for row in rows {
        //let title: String = row.get("title");
        //let source: String = row.get("source");
        let id: u16 = row.get("id");
        //let source: String = row.get("source");
        movies.push(format!("/refreshdata/{}", id)); 
        

    }

    movies.join("\n")
}

// Fallback to serve the `index.html` file when no route matches
#[delete("/movie/<id>")]
async fn delete_movie(state: &State<AppState>, id: &str) ->  &'static str {
    let db: &sqlx::Pool<sqlx::Sqlite> = &state.db_pool;
    sqlx::query("DELETE FROM movies where imdbid=?")
    .bind(id)
    .execute(db)
    .await
    .expect("Failed to delete movie");
    "Movie Deleted"
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

async fn update_metadata(state: &State<AppState>, title: String, source: String) {
    let db: &sqlx::Pool<sqlx::Sqlite> = &state.db_pool;
    let meta = fetch_metadata(&state, 
        title.as_str()).await;

    match meta {
        Ok(meta) => {
            //println!("{:?}", meta);
            let mut metadata: MovieMetadata = serde_json::from_str(&meta)
                .expect("ERROR");

            //let metadata_json = serde_json::to_string(&meta).expect("Failed to serialize metadata");
            // println!("{}", metadata_json);
            if metadata.response == "False" {
                metadata.title = Some(String::from(title.as_str())); 
            } 
            metadata.source = Some(source.clone());
            metadata.plot = escape_plot(metadata.plot);
            metadata.title = escape_plot(metadata.title);
            match sqlx::query("INSERT INTO movies (title, imdbid, year, metadata, source) VALUES (?, ?, ?, ?, ?)")
                .bind(title.clone())
                .bind(metadata.imdb_id.clone())
                .bind(metadata.year.clone())
                .bind(serde_json::json!(metadata))
                .bind(source.clone())
                .execute(db)
                .await
        {
            Ok(_) => {
                if let Some(year) = metadata.year {
                    println!("Movie inserted successfully: {} ({})", title, year);
                } else {
                    println!("Movie inserted successfully: {} (Year not provided)", title);
                }
            }            
            Err(sqlx::Error::Database(err)) if err.code().unwrap_or_default() == "2067" => {
                match sqlx::query("UPDATE movies SET metadata=? where imdbid=?")
                    .bind(serde_json::json!(metadata))
                    .bind(metadata.imdb_id)
                    //.bind(metadata.year.clone())
                
                    .execute(db)
                    .await
                    {
                        Ok(opt) => 
                            println!("Movie({})  updated successfully: {} ({:?})", opt.rows_affected() , 
                                title, metadata.year.clone().unwrap()),
                        Err(err) => eprintln!("Unexpected error: {}", err),
                    };
                println!("Duplicate movie found: {} ({:?})", title, metadata.year);
            }
            Err(err) => eprintln!("Unexpected error: {}", err),
        };
            

            

            
        
            //println!("Done Update: {}", title);
        }
        Err(e) => {
            sqlx::query("INSERT INTO movies (title, source) VALUES (?, ?)")
                .bind(title.clone())
                .bind(source.clone())
                .execute(db)
                .await
                .expect("Failed to insert movie");
            eprintln!("Failed to fetch metadata for {}: {:?}", title, e);
            () // Wrap it in a vector to return it as JSON
        }
    }
}

// Handler for file uploads
#[post("/upload", data = "<data>")]
async fn upload_movies(state: &State<AppState>, data: String) -> &'static str {
    
    // Parse JSON data
    let parsed_data: serde_json::Value = match serde_json::from_str(&data) {
        Ok(json) => json,
        Err(_) => {
            return "Invalid JSON format";
        }
    };

    let source = match parsed_data["source"].as_str() {
        Some(value) => value.to_string(), // Convert &str to String
        None => "default_value".to_string(), // Provide a default value or handle the missing case
    };
    println!("Source: {:?}", source.clone());

    if let Some(titles) = parsed_data["titles"].as_array() {

        for value in titles {
            let title = match value.as_str() {
                Some(value) => value.to_string(), // Convert &str to String
                None => "default_value".to_string(), // Provide a default value or handle the missing case
            };
            update_metadata(state, title, source.clone()).await;
               
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
        .mount("/", routes![get_movies, upload_movies, delete_metadata,
		 get_missing_data, saved_file, save_file, 
         refresh_metadata,get_movie,
         delete_movie])
        .mount("/foo", routes![serve_index])
        .mount("/", FileServer::from("static")) // Serve static files
        .attach(Template::fairing())

}
