# Movie Database Application (Rust + Rocket + SQLite)

## Overview
This is a web-based movie database application built using Rust with the Rocket framework. The application allows users to manage a list of movies, fetch metadata from external sources (such as the OMDb API), and display relevant details.

## Features
- Web-based interface using Rocket and HTMX
- SQLite as the backend database
- Fetch movie metadata from the OMDb API
- Store and retrieve movie details
- Dynamic interactions without full-page reloads using HTMX

## Frequently Asked Questions

### 1. Does this run as a web client?
Yes, this application provides a web interface accessible through a browser. The frontend communicates with the backend server via HTTP requests.

### 2. If so, are you running a server on your local machine?
Yes, the Rocket framework runs a local web server on your machine. By default, the server starts at `http://127.0.0.1:8000`, where you can interact with the application.

## Installation

### Prerequisites
Ensure you have the following installed:
- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- SQLite (for database storage)
- `cargo` (Rust package manager)

### Setup Instructions
1. **Clone the Repository**
   ```sh
   git clone https://github.com/yourusername/movie-db-rust.git
   cd movie-db-rust
   ```
2. **Install Dependencies**
   ```sh
   cargo build
   ```
3. **Run the Application**
   ```sh
   cargo run
   ```
4. **Access the Web Interface**
   Open your browser and go to:
   ```
   http://127.0.0.1:8000
   ```

## API Endpoints

| Method | Endpoint           | Description |
|--------|-------------------|-------------|
| GET    | `/movies`         | Fetch all movies |
| POST   | `/movies/upload`  | Upload a list of movies |
| GET    | `/movies/:id`     | Fetch movie details by ID |

## Screenshots
To help visualize the interface, here are some example screenshots:

*📌 [Add screenshots here once available]*

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you'd like to change.

## License
This project is licensed under the MIT License.

