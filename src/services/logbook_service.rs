use chrono::Utc;
use sha2::{Digest, Sha256};
use tokio::fs;

use crate::{
    auth::CurrentUser,
    error::AppError,
    models::{UserRole, VisitDetailView},
    services::visit_service::SELECT_VISIT_DETAIL_SQL,
    AppState,
};

pub fn validate_safe_id(id: &str) -> Result<(), AppError> {
    if id.is_empty() || id.len() > 64 || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(AppError::BadRequest("Format ID kunjungan tidak valid".into()));
    }
    Ok(())
}

pub fn detect_image_format(bytes: &[u8]) -> Result<(&'static str, &'static str), AppError> {
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok(("image/webp", "webp"));
    }
    if bytes.len() >= 3 && bytes[0..3] == [0xFF, 0xD8, 0xFF] {
        return Ok(("image/jpeg", "jpg"));
    }
    if bytes.len() >= 8 && bytes[0..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        return Ok(("image/png", "png"));
    }
    Err(AppError::BadRequest(
        "File tidak valid: wajib berupa foto WebP, JPEG, atau PNG".to_string(),
    ))
}

pub async fn complete_visit_with_logbook(
    state: &AppState,
    admin: &CurrentUser,
    visit_id: &str,
    photo_bytes: Vec<u8>,
    meeting_notes: Option<String>,
) -> Result<VisitDetailView, AppError> {
    admin.require_role(UserRole::Admin)?;
    validate_safe_id(visit_id)?;

    if photo_bytes.is_empty() {
        return Err(AppError::BadRequest("Foto pertemuan wajib diunggah".into()));
    }
    if photo_bytes.len() > state.config.max_photo_upload_bytes {
        return Err(AppError::BadRequest(
            "Ukuran foto melebihi batas maksimal 2 MB".into(),
        ));
    }

    let (_mime, ext) = detect_image_format(&photo_bytes)?;

    let now = Utc::now();
    let year_month = now.format("%Y/%m").to_string();
    let now_iso = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let filename = format!("{}.{}", visit_id, ext);
    let photo_rel_path = format!("{}/{}", year_month, filename);
    let rel_dir = state.photo_base_dir.join(&year_month);
    let full_disk_path = state.photo_base_dir.join(&photo_rel_path);

    let mut hasher = Sha256::new();
    hasher.update(&photo_bytes);
    let photo_sha256 = hex::encode(hasher.finalize());
    let photo_size_bytes = photo_bytes.len() as i64;

    // 1. Mulai transaksi DB terlebih dahulu sebelum menyentuh disk
    let mut tx = state.db.begin().await?;

    let update_res = sqlx::query(
        "UPDATE visit_requests
         SET current_status = 'COMPLETED', updated_at = ?1
         WHERE id = ?2 AND current_status = 'SANTRI_CONFIRMED'"
    )
    .bind(&now_iso)
    .bind(visit_id)
    .execute(&mut *tx)
    .await?;

    if update_res.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "Tiket harus berstatus SANTRI_CONFIRMED sebelum mengisi Logbook".into(),
        ));
    }

    let logbook_id = uuid::Uuid::now_v7().to_string();
    sqlx::query(
        "INSERT INTO visit_logbooks (
            id, visit_id, captured_by_admin_id, photo_rel_path,
            photo_size_bytes, photo_sha256, meeting_notes, meeting_timestamp
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )
    .bind(&logbook_id)
    .bind(visit_id)
    .bind(&admin.id)
    .bind(&photo_rel_path)
    .bind(photo_size_bytes)
    .bind(&photo_sha256)
    .bind(meeting_notes.as_deref().map(str::trim))
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO visit_status_logs (visit_id, from_status, to_status, actor_id, public_note, created_at)
         VALUES (?1, 'SANTRI_CONFIRMED', 'COMPLETED', ?2, 'Pertemuan Wali Santri & Santri selesai dan terdokumentasi di Logbook', ?3)"
    )
    .bind(visit_id)
    .bind(&admin.id)
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    // 2. Hanya tulis ke disk setelah DB update tervalidasi; jika gagal, tx otomatis rollback
    fs::create_dir_all(&rel_dir)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Gagal membuat folder foto: {}", e)))?;

    fs::write(&full_disk_path, &photo_bytes)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Gagal menyimpan file foto: {}", e)))?;

    // 3. Commit transaksi setelah file berhasil ditulis ke disk
    tx.commit().await?;

    let query_sql = format!("{} WHERE v.id = ?1", SELECT_VISIT_DETAIL_SQL);
    let detail: VisitDetailView = sqlx::query_as(&query_sql)
        .bind(visit_id)
        .fetch_one(&state.db)
        .await?;

    let wa_msg = format!(
        "Alhamdulillah, kunjungan sambang untuk {} ({}) telah selesai dan tercatat di Logbook. Dokumentasi foto pertemuan dapat dilihat di akun SiPesantren Anda.",
        detail.santri_name, detail.ticket_code
    );
    state
        .notifier
        .notify_wali(Some(&detail.id), &detail.wali_phone, "COMPLETED", &wa_msg)
        .await;

    state.sse.emit_dormitory_event("dormitory_update", &detail);
    state.sse.emit_public_flow_event(&detail);

    Ok(detail)
}
