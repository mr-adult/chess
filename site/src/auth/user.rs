use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::AppState;

/// IMPORTANT: Do not put the password in this struct.
/// This struct is sent to the client.
#[derive(Clone, FromRow, Debug, Serialize)]
pub struct User {
    pub id: Uuid,
    pub user_name: String,
}

/// This should never be send anywhere. It comes in from
/// the client as a create user request.
#[derive(Deserialize)]
pub struct UserForCreation {
    pub user_name: String,
    pub password_plain_text: String,
}

struct UserForInsertIntoDB {
    user_name: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct UserForLogin {
    pub id: Uuid,
    pub user_name: String,

    /// The password in the form of {scheme ID}_{hashed password}
    pub password_hash: Option<String>,
    pub password_salt: Uuid,
    pub token_salt: Uuid,
}

impl UserForLogin {
    async fn get(pool: &mut PgPool, id: Uuid) -> Option<UserForLogin> {
        /* sqlx::query_as!(
            UserForLogin,
            "SELECT * FROM users WHERE id = $1",
            id
        ).fetch_one(pool)
            .await?*/
        todo!();
    }
}

#[derive(Clone, FromRow, Debug)]
pub struct UserForAuth {
    pub id: Uuid,
    pub user_name: String,

    pub token_salt: Uuid,
}
