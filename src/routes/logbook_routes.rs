use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, HeaderName, StatusCode},
    response::IntoResponse,
    Json,
};
use tokio::fs;

use crate::{
    auth::CurrentUser,
    error::AppError,
    models::UserRole,
    services::logbook_service::{
        complete_visit_with_logbook, detect_image_format, validate_safe_id,
    },
    AppState,
};

pub async fn handle_upload_logbook(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(visit_id): Path<String>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    user.require_role(UserRole::Admin)?;
    validate_safe_id(&visit_id)?;

    let mut photo_bytes: Option<Vec<u8>> = None;
    let mut meeting_notes: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "photo" {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("Gagal membaca byte foto: {}", e)))?;
            photo_bytes = Some(data.to_vec());
        } else if name == "meeting_notes" {
            let txt = field
                .text()
                .await
                .map_err(|e| AppError::BadRequest(format!("Gagal membaca catatan: {}", e)))?;
            if !txt.trim().is_empty() {
                meeting_notes = Some(txt);
            }
        }
    }

    let bytes = photo_bytes.ok_or_else(|| AppError::BadRequest("Field 'photo' wajib dikirim".into()))?;
    let detail = complete_visit_with_logbook(&state, &user, &visit_id, bytes, meeting_notes).await?;

    Ok((
        StatusCode::OK,
        [(HeaderName::from_static("hx-trigger"), "visitChanged")],
        Json(detail),
    ))
}

pub async fn get_protected_logbook_photo(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(visit_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    validate_safe_id(&visit_id)?;

    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT v.wali_id, l.photo_rel_path
         FROM visit_requests v
         JOIN visit_logbooks l ON l.visit_id = v.id
         WHERE v.id = ?1"
    )
    .bind(&visit_id)
    .fetch_optional(&state.db)
    .await?;

    let (owner_wali_id, photo_rel_path) =
        row.ok_or_else(|| AppError::NotFound("Foto logbook tidak ditemukan".into()))?;

    let is_authorized = match user.role {
        UserRole::Admin => true,
        UserRole::Wali => user.id == owner_wali_id,
        UserRole::Musyrif | UserRole::Asatidz => false,
    };

    if !is_authorized {
        return Err(AppError::Forbidden);
    }

    if photo_rel_path.contains("..") {
        return Err(AppError::Forbidden);
    }

    let full_path = state.photo_base_dir.join(&photo_rel_path);
    let bytes = fs::read(&full_path)
        .await
        .map_err(|_| AppError::NotFound("File foto tidak ditemukan di penyimpanan".into()))?;

    let (mime_type, _) = detect_image_format(&bytes).unwrap_or(("image/webp", "webp"));

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, mime_type.parse().unwrap());
    headers.insert(
        header::CACHE_CONTROL,
        "private, no-transform, max-age=86400".parse().unwrap(),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        "nosniff".parse().unwrap(),
    );

    Ok((StatusCode::OK, headers, bytes))
}
