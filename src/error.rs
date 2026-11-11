use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Autentikasi diperlukan: silakan login terlebih dahulu")]
    Unauthorized,

    #[error("Akses ditolak: Anda tidak memiliki izin untuk sumber daya ini")]
    Forbidden,

    #[error("Data tidak ditemukan: {0}")]
    NotFound(String),

    #[error("Input atau status tidak valid: {0}")]
    BadRequest(String),

    #[error("Konflik data atau status jadwal: {0}")]
    Conflict(String),

    #[error("Kesalahan database internal")]
    Database(#[from] sqlx::Error),

    #[error("Kesalahan sistem internal")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Database(e) => {
                tracing::error!("SQLx database error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let message = self.to_string();
        let body = Json(json!({
            "error": true,
            "message": message
        }));

        (status, body).into_response()
    }
}

/// Pembungkus template Askama ke Axum Response tanpa ketergantungan askama_axum
pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => {
                tracing::error!("Askama template render error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Kesalahan saat merender tampilan halaman",
                )
                    .into_response()
            }
        }
    }
}
