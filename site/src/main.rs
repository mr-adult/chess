use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Router,
};
use html_to_string_macro::html;
use http::Method;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use sqlx::{postgres::PgPoolOptions, PgPool};

mod api;
mod auth;
mod chess_html;

const TOKEN_EXPIRATION_TIME_SECS: usize = 1800; // 30 mins

/// Making this a function so I can interject a .env file value later on.
pub fn get_token_expiration_secs() -> usize {
    TOKEN_EXPIRATION_TIME_SECS
}

#[tokio::main]
async fn main() {
    println!("Starting...");

    // First, parse the .env file for our environment setup.
    dotenvy::dotenv().ok();

    // let pool = start_db().await;

    // Set up the routes for our application
    let app = Router::new()
        .route("/", get(home))
        .route("/login", post(auth::login_handler))
        .nest_service("/api/v0", api::create_api_router())
        .nest_service("/styles", ServeDir::new("src/styles"))
        .nest_service("/scripts", ServeDir::new("src/scripts"))
        .layer(
            // Add CORS so it doesn't block our requests from the browser
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any),
        )
        .layer(CookieManagerLayer::new())
        // Attach our connection pool to every endpoint so the endpoints can query the DB.
        .with_state(AppState {
            // pgpool: pool
        });

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

#[derive(Clone)] // This will be cloned per-request, so no expensive to copy data should be introduced here.
pub(crate) struct AppState {
    // pub(crate) pgpool: PgPool
}

#[allow(unused)]
async fn start_db() -> PgPool {
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

    return pool;
}

async fn home(state: State<AppState>) -> Html<String> {
    Html(html! {
        <!DOCTYPE html>
        <head>
            <title>"Chess"</title>
            <meta charset="UTF-8" />
            <meta name="description" content="A chess website" />
            <link rel="stylesheet" href="/styles/app.css" />
            <script src="/scripts/app.js"></script>
        </head>
        <body>
            {chess_html::new_game().await.0}
        </body>
    })
}
