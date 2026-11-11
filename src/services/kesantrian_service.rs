use serde::Deserialize;
use crate::{
    auth::CurrentUser,
    error::AppError,
    models::{validate_iso_date, AttendanceLogView, SantriNoteView},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct RecordAttendanceInput {
    pub santri_id: String,
    pub attendance_date: String,
    pub activity_type: String,
    pub status: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSantriNoteInput {
    pub santri_id: String,
    pub note_date: String,
    pub category: String,
    pub title: String,
    pub description: String,
}

pub async fn upsert_attendance_log(
    state: &AppState,
    actor: &CurrentUser,
    input: RecordAttendanceInput,
) -> Result<AttendanceLogView, AppError> {
    let clean_date = validate_iso_date(&input.attendance_date)?;

    let dorm_id: Option<String> = sqlx::query_scalar(
        "SELECT dormitory_id FROM santri WHERE id = ?1 AND is_active = 1"
    )
    .bind(&input.santri_id)
    .fetch_optional(&state.db)
    .await?;

    let dorm_id = dorm_id.ok_or_else(|| AppError::NotFound("Santri tidak ditemukan".into()))?;
    actor.require_staff_for_dorm(&dorm_id)?;

    if !matches!(
        input.activity_type.as_str(),
        "SHOLAT_JAMAAH" | "KBM_DINIYAH" | "HALAQAH_TAHFIDZ" | "ASRAMA_MALAM"
    ) {
        return Err(AppError::BadRequest("Jenis kegiatan absensi tidak valid".into()));
    }
    if !matches!(input.status.as_str(), "HADIR" | "SAKIT" | "IZIN" | "ALFA") {
        return Err(AppError::BadRequest("Status kehadiran tidak valid".into()));
    }

    let id = uuid::Uuid::now_v7().to_string();
    sqlx::query(
        "INSERT INTO attendance_logs (id, santri_id, attendance_date, activity_type, status, note, recorded_by_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(santri_id, attendance_date, activity_type) DO UPDATE SET
            status = excluded.status,
            note = excluded.note,
            recorded_by_id = excluded.recorded_by_id"
    )
    .bind(&id)
    .bind(&input.santri_id)
    .bind(&clean_date)
    .bind(&input.activity_type)
    .bind(&input.status)
    .bind(input.note.as_deref().map(str::trim))
    .bind(&actor.id)
    .execute(&state.db)
    .await?;

    let row: AttendanceLogView = sqlx::query_as(
        "SELECT a.id, a.santri_id, s.full_name AS santri_name, a.attendance_date,
                a.activity_type, a.status, a.note, u.full_name AS recorded_by_name, a.created_at
         FROM attendance_logs a
         JOIN santri s ON s.id = a.santri_id
         JOIN users u ON u.id = a.recorded_by_id
         WHERE a.santri_id = ?1 AND a.attendance_date = ?2 AND a.activity_type = ?3"
    )
    .bind(&input.santri_id)
    .bind(&clean_date)
    .bind(&input.activity_type)
    .fetch_one(&state.db)
    .await?;

    Ok(row)
}

pub async fn create_santri_note(
    state: &AppState,
    actor: &CurrentUser,
    input: CreateSantriNoteInput,
) -> Result<SantriNoteView, AppError> {
    let clean_date = validate_iso_date(&input.note_date)?;

    let dorm_id: Option<String> = sqlx::query_scalar(
        "SELECT dormitory_id FROM santri WHERE id = ?1 AND is_active = 1"
    )
    .bind(&input.santri_id)
    .fetch_optional(&state.db)
    .await?;

    let dorm_id = dorm_id.ok_or_else(|| AppError::NotFound("Santri tidak ditemukan".into()))?;
    actor.require_staff_for_dorm(&dorm_id)?;

    if !matches!(
        input.category.as_str(),
        "PRESTASI" | "PEMBINAAN_ADAB" | "IZIN_KESEHATAN_PULANG"
    ) {
        return Err(AppError::BadRequest("Kategori catatan tidak valid".into()));
    }
    if input.title.trim().is_empty() || input.description.trim().is_empty() {
        return Err(AppError::BadRequest("Judul dan deskripsi catatan wajib diisi".into()));
    }

    let id = uuid::Uuid::now_v7().to_string();
    sqlx::query(
        "INSERT INTO santri_notes (id, santri_id, note_date, category, title, description, recorded_by_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    )
    .bind(&id)
    .bind(&input.santri_id)
    .bind(&clean_date)
    .bind(&input.category)
    .bind(input.title.trim())
    .bind(input.description.trim())
    .bind(&actor.id)
    .execute(&state.db)
    .await?;

    let view: SantriNoteView = sqlx::query_as(
        "SELECT n.id, n.santri_id, s.full_name AS santri_name, n.note_date,
                n.category, n.title, n.description, u.full_name AS recorded_by_name, n.created_at
         FROM santri_notes n
         JOIN santri s ON s.id = n.santri_id
         JOIN users u ON u.id = n.recorded_by_id
         WHERE n.id = ?1"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await?;

    Ok(view)
}
