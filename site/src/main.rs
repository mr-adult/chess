use axum::{
    extract::State,
    response::Html,
    routing::{get},
    Router,
};
use chess_core::Board;
use chess_html::render_gameboard_full_page;
use http::Method;
use tower_cookies::CookieManagerLayer;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};

mod api;
mod chess_html;
mod common;

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

    // Set up the routes for our application
    let app = Router::new()
        .route("/", get(home))
        .nest_service("/api/v0", api::create_api_router())
        .nest_service("/html/v0", chess_html::create_ssr_router())
        .nest_service("/styles", ServeDir::new("src/styles"))
        .nest_service("/scripts", ServeDir::new("src/scripts"))
        .nest_service("/favicon.ico", ServeFile::new("src/favicon.ico"))
        .layer(
            // Add CORS so it doesn't block our requests from the browser
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_origin(Any),
        )
        .layer(CookieManagerLayer::new())
        // Attach our connection pool to every endpoint so the endpoints can query the DB.
        .with_state(AppState {});

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
pub(crate) struct AppState {}

async fn home(state: State<AppState>) -> Html<String> {
    render_gameboard_full_page(&Board::default())
}
