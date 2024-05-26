use axum::{extract::State, Json};
use tower_cookies::{Cookie, Cookies};

use crate::AppState;

mod user;
mod crypt;

pub fn get_auth_token_cookie_name() -> String {
    return "auth_token".to_string()
}

pub(crate) async fn login_handler(
    state: State<AppState>,
    cookies: Cookies,
) -> Result<Json<()>, ()> {
    cookies.add(Cookie::new(get_auth_token_cookie_name(), "1"));
    Ok(Json(()))
}