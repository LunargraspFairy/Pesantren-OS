use axum::{
    extract::{Form, State},
    response::Redirect,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use serde::Deserialize;

use crate::{
    auth::{
        create_user_session, hash_session_token, verify_password, SESSION_COOKIE_NAME,
    },
    error::AppError,
    models::{User, UserRole},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub phone_number: String,
    pub password: String,
}

pub async fn handle_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<(CookieJar, Redirect), AppError> {
    let phone = form.phone_number.trim();
    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, full_name, phone_number, password_hash, role, dormitory_id, is_active, created_at
         FROM users
         WHERE phone_number = ?1 AND is_active = 1"
    )
    .bind(phone)
    .fetch_optional(&state.db)
    .await?;

    let Some(user) = user else {
        return Err(AppError::Unauthorized);
    };

    if !verify_password(&form.password, &user.password_hash) {
        return Err(AppError::Unauthorized);
    }

    let raw_token =
        create_user_session(&state.db, &user.id, state.config.session_ttl_hours).await?;

    let cookie = Cookie::build((SESSION_COOKIE_NAME, raw_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build();

    let dest = match UserRole::from_db(&user.role) {
        Some(UserRole::Admin) => "/admin",
        Some(UserRole::Wali) => "/wali",
        Some(UserRole::Musyrif) => "/musyrif/board",
        Some(UserRole::Asatidz) => "/asatidz",
        None => "/alur",
    };

    Ok((jar.add(cookie), Redirect::to(dest)))
}

pub async fn handle_logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), AppError> {
    if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
        let token_hash = hash_session_token(cookie.value());
        let _ = sqlx::query("DELETE FROM sessions WHERE token_hash = ?1")
            .bind(&token_hash)
            .execute(&state.db)
            .await;
    }

    let removal = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .build();

    Ok((jar.remove(removal), Redirect::to("/login")))
}
