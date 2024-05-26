use axum::{
    extract::State,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use http::Method;
use tower_http::cors::{Any, CorsLayer};

use sqlx::{postgres::PgPoolOptions, PgPool};

#[tokio::main]
async fn main() {
    println!("Starting...");

    // First, parse the .env file for our environment setup.
    dotenvy::dotenv().ok();

    // We create a single connection pool for SQLx that's shared across the whole application.
    // This saves us from opening a new connection for every API call, which is wasteful.
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("{}", database_url);
    let pool = PgPoolOptions::new()
        // The default connection limit for a Postgres server is 100 connections, minus 3 for superusers.
        // We should leave some connections available for manual access.
        //
        // If you're deploying your application with multiple replicas, then the total
        // across all replicas should not exceed the Postgres connection limit.
        .max_connections(10)
        .connect(&database_url)
        .await
        .unwrap_or_else(|err| panic!("Could not connect to dabase_url. Error: \n{}", err));

    // Run any SQL migrations to get the DB into the correct state
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap_or_else(|err| panic!("Failed to migrate the database. Error: \n{}", err));

    // Set up the routes for our application
    let app = Router::new()
        .route("/", get(hello_world))
        .layer(
            // Add CORS so it doesn't block our requests from the browser
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any),
        )
        // Attach our connection pool to every endpoint so the endpoints can query the DB.
        .with_state(pool);

    // Bind to port 8080
    let listener = tokio::net::TcpListener::bind("[::]:8080")
        .await
        .unwrap_or_else(|err| panic!("Failed to initialize TCP listener. Error: \n{}", err));

    // Serve is an infinite async function, so we have to report that we're listening before awaiting.
    println!("Now listening on port 8080");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|err| panic!("Failed to start app. Error: \n{}", err));
}

async fn hello_world(state: State<PgPool>) -> Html<String> {
    return Html("Hello, world!".to_string());
}
