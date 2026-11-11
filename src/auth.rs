use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use rand_core::RngCore;
use sha2::{Digest, Sha256};

use crate::{
    error::AppError,
    models::{User, UserRole},
    AppState,
};

pub const SESSION_COOKIE_NAME: &str = "sisambang_session";

pub fn hash_password(plain: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Password hash error: {}", e)))
}

pub fn verify_password(plain: &str, hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(plain.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn generate_session_token() -> (String, String) {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let raw_token = hex::encode(bytes);
    let token_hash = hash_session_token(&raw_token);
    (raw_token, token_hash)
}

pub fn hash_session_token(raw_token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    hex::encode(hasher.finalize())
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: String,
    pub full_name: String,
    pub phone_number: String,
    pub role: UserRole,
    pub dormitory_id: Option<String>,
}

impl CurrentUser {
    pub fn require_role(&self, expected: UserRole) -> Result<(), AppError> {
        if self.role == expected {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }

    pub fn require_admin_or_asatidz(&self) -> Result<(), AppError> {
        match self.role {
            UserRole::Admin | UserRole::Asatidz => Ok(()),
            _ => Err(AppError::Forbidden),
        }
    }

    pub fn require_staff_for_dorm(&self, target_dorm_id: &str) -> Result<(), AppError> {
        match self.role {
            UserRole::Admin | UserRole::Asatidz => Ok(()),
            UserRole::Musyrif => {
                if self.dormitory_id.as_deref() == Some(target_dorm_id) {
                    Ok(())
                } else {
                    Err(AppError::Forbidden)
                }
            }
            UserRole::Wali => Err(AppError::Forbidden),
        }
    }

    pub fn require_admin_or_musyrif_for_dorm(&self, target_dorm_id: &str) -> Result<(), AppError> {
        match self.role {
            UserRole::Admin => Ok(()),
            UserRole::Musyrif => {
                if self.dormitory_id.as_deref() == Some(target_dorm_id) {
                    Ok(())
                } else {
                    Err(AppError::Forbidden)
                }
            }
            _ => Err(AppError::Forbidden),
        }
    }

    pub fn home_redirect_path(&self) -> &'static str {
        match self.role {
            UserRole::Admin => "/admin",
            UserRole::Wali => "/wali",
            UserRole::Musyrif => "/musyrif/board",
            UserRole::Asatidz => "/asatidz",
        }
    }
}

#[async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let Some(cookie) = jar.get(SESSION_COOKIE_NAME) else {
            return Err(AppError::Unauthorized);
        };

        let token_hash = hash_session_token(cookie.value());
        let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let user: Option<User> = sqlx::query_as::<_, User>(
            "SELECT u.id, u.full_name, u.phone_number, u.password_hash, u.role, u.dormitory_id, u.is_active, u.created_at
             FROM sessions s
             JOIN users u ON u.id = s.user_id
             WHERE s.token_hash = ?1 AND s.expires_at > ?2 AND u.is_active = 1"
        )
        .bind(&token_hash)
        .bind(&now_iso)
        .fetch_optional(&state.db)
        .await?;

        let user = user.ok_or(AppError::Unauthorized)?;
        let role = UserRole::from_db(&user.role).ok_or(AppError::Unauthorized)?;

        Ok(CurrentUser {
            id: user.id,
            full_name: user.full_name,
            phone_number: user.phone_number,
            role,
            dormitory_id: user.dormitory_id,
        })
    }
}

pub async fn create_user_session(
    db: &sqlx::SqlitePool,
    user_id: &str,
    ttl_hours: i64,
) -> Result<String, AppError> {
    let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let _ = sqlx::query("DELETE FROM sessions WHERE expires_at <= ?1")
        .bind(&now_iso)
        .execute(db)
        .await;

    let (raw_token, token_hash) = generate_session_token();
    let expires_at = (Utc::now() + Duration::hours(ttl_hours))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    sqlx::query("INSERT INTO sessions (token_hash, user_id, expires_at) VALUES (?1, ?2, ?3)")
        .bind(&token_hash)
        .bind(user_id)
        .bind(&expires_at)
        .execute(db)
        .await?;

    Ok(raw_token)
}
