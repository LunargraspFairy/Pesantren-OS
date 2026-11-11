use chrono::Utc;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    auth::CurrentUser,
    error::AppError,
    models::{validate_hhmm_time, validate_iso_date, UserRole, VisitDetailView},
    AppState,
};

pub const SELECT_VISIT_DETAIL_SQL: &str = r#"
SELECT
    v.id,
    v.ticket_code,
    v.wali_id,
    u.full_name AS wali_name,
    u.phone_number AS wali_phone,
    v.santri_id,
    s.full_name AS santri_name,
    s.nis AS santri_nis,
    s.dormitory_id,
    d.name AS dormitory_name,
    s.room_name,
    v.requested_date,
    v.requested_session,
    v.scheduled_start,
    v.scheduled_end,
    v.meeting_spot,
    v.visitor_count,
    v.has_titipan,
    v.titipan_description,
    v.current_status,
    v.santri_notified_by_musyrif,
    v.titipan_confirmed_to_wali,
    v.cancel_reason,
    CASE WHEN l.id IS NOT NULL THEN 1 ELSE 0 END AS has_logbook,
    l.meeting_timestamp,
    l.meeting_notes,
    v.created_at,
    v.updated_at
FROM visit_requests v
JOIN users u ON u.id = v.wali_id
JOIN santri s ON s.id = v.santri_id
JOIN dormitories d ON d.id = s.dormitory_id
LEFT JOIN visit_logbooks l ON l.visit_id = v.id
"#;

pub fn deserialize_checkbox_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrString {
        Bool(bool),
        Str(String),
        Int(i64),
    }

    match Option::<BoolOrString>::deserialize(deserializer)? {
        Some(BoolOrString::Bool(b)) => Ok(b),
        Some(BoolOrString::Int(i)) => Ok(i != 0),
        Some(BoolOrString::Str(s)) => {
            let v = s.trim().to_ascii_lowercase();
            Ok(v == "true" || v == "1" || v == "on" || v == "yes")
        }
        None => Ok(false),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateVisitInput {
    pub santri_id: String,
    pub requested_date: String,
    pub requested_session: String,
    pub visitor_count: i64,
    #[serde(default, deserialize_with = "deserialize_checkbox_bool")]
    pub has_titipan: bool,
    pub titipan_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleVisitInput {
    pub scheduled_start: String,
    pub scheduled_end: String,
    pub meeting_spot: String,
}

#[derive(Debug, Deserialize)]
pub struct CancelVisitInput {
    pub cancel_reason: String,
}

pub async fn fetch_visit_detail(
    db: &SqlitePool,
    visit_id: &str,
) -> Result<VisitDetailView, AppError> {
    let sql = format!("{} WHERE v.id = ?1", SELECT_VISIT_DETAIL_SQL);
    sqlx::query_as::<_, VisitDetailView>(&sql)
        .bind(visit_id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Tiket {} tidak ditemukan", visit_id)))
}

pub async fn create_visit_request(
    state: &AppState,
    user: &CurrentUser,
    input: CreateVisitInput,
) -> Result<VisitDetailView, AppError> {
    user.require_role(UserRole::Wali)?;

    let req_date = validate_iso_date(&input.requested_date)?;
    if !matches!(input.requested_session.as_str(), "PAGI" | "SIANG" | "SORE") {
        return Err(AppError::BadRequest("Sesi kunjungan harus PAGI, SIANG, atau SORE".into()));
    }
    if !(1..=10).contains(&input.visitor_count) {
        return Err(AppError::BadRequest("Jumlah pengunjung harus antara 1 sampai 10 orang".into()));
    }

    let cleaned_titipan = if input.has_titipan {
        let desc = input
            .titipan_description
            .as_deref()
            .map(str::trim)
            .unwrap_or("");
        if desc.is_empty() || desc.len() > 500 {
            return Err(AppError::BadRequest(
                "Deskripsi barang titipan wajib diisi (maks 500 karakter)".into(),
            ));
        }
        Some(desc.to_string())
    } else {
        None
    };

    let owns_santri: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM santri WHERE id = ?1 AND primary_wali_id = ?2 AND is_active = 1"
    )
    .bind(&input.santri_id)
    .bind(&user.id)
    .fetch_one(&state.db)
    .await?;

    if owns_santri == 0 {
        return Err(AppError::Forbidden);
    }

    let active_dup: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM visit_requests
         WHERE santri_id = ?1 AND requested_date = ?2
         AND current_status NOT IN ('CANCELLED', 'COMPLETED')"
    )
    .bind(&input.santri_id)
    .bind(&req_date)
    .fetch_one(&state.db)
    .await?;

    if active_dup > 0 {
        return Err(AppError::Conflict(
            "Santri sudah memiliki jadwal sambang aktif pada tanggal tersebut".into(),
        ));
    }

    let id = uuid::Uuid::now_v7().to_string();
    let suffix = id[id.len().saturating_sub(4)..].to_uppercase();
    let ticket_code = format!("SMB-{}-{}", req_date.replace('-', ""), suffix);
    let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let mut tx = state.db.begin().await?;

    sqlx::query(
        "INSERT INTO visit_requests (
            id, ticket_code, wali_id, santri_id, requested_date, requested_session,
            visitor_count, has_titipan, titipan_description, current_status, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'REQUESTED', ?10, ?10)"
    )
    .bind(&id)
    .bind(&ticket_code)
    .bind(&user.id)
    .bind(&input.santri_id)
    .bind(&req_date)
    .bind(&input.requested_session)
    .bind(input.visitor_count)
    .bind(if input.has_titipan { 1 } else { 0 })
    .bind(&cleaned_titipan)
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO visit_status_logs (visit_id, from_status, to_status, actor_id, public_note, created_at)
         VALUES (?1, NULL, 'REQUESTED', ?2, 'Pengajuan sambang dibuat oleh Wali Santri', ?3)"
    )
    .bind(&id)
    .bind(&user.id)
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let detail = fetch_visit_detail(&state.db, &id).await?;
    state.sse.emit_public_flow_event(&detail);
    Ok(detail)
}

pub async fn schedule_visit(
    state: &AppState,
    user: &CurrentUser,
    visit_id: &str,
    input: ScheduleVisitInput,
) -> Result<VisitDetailView, AppError> {
    user.require_role(UserRole::Admin)?;

    let start_hhmm = validate_hhmm_time(&input.scheduled_start)?;
    let end_hhmm = validate_hhmm_time(&input.scheduled_end)?;
    if start_hhmm >= end_hhmm {
        return Err(AppError::BadRequest(
            "Jam selesai harus lebih besar dari jam mulai".into(),
        ));
    }

    let spot = input.meeting_spot.trim();
    if spot.is_empty() {
        return Err(AppError::BadRequest("Lokasi ruang/gazebo temu wajib diisi".into()));
    }

    let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let note = format!("Jadwal ditetapkan pukul {}-{} di {}", start_hhmm, end_hhmm, spot);

    let mut tx = state.db.begin().await?;

    let res = sqlx::query(
        "UPDATE visit_requests
         SET scheduled_start = ?1, scheduled_end = ?2, meeting_spot = ?3,
             current_status = 'SCHEDULED', updated_at = ?4
         WHERE id = ?5 AND current_status = 'REQUESTED'"
    )
    .bind(&start_hhmm)
    .bind(&end_hhmm)
    .bind(spot)
    .bind(&now_iso)
    .bind(visit_id)
    .execute(&mut *tx)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::Conflict("Tiket tidak dalam status REQUESTED".into()));
    }

    sqlx::query(
        "INSERT INTO visit_status_logs (visit_id, from_status, to_status, actor_id, public_note, created_at)
         VALUES (?1, 'REQUESTED', 'SCHEDULED', ?2, ?3, ?4)"
    )
    .bind(visit_id)
    .bind(&user.id)
    .bind(&note)
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let detail = fetch_visit_detail(&state.db, visit_id).await?;
    let wa_msg = format!(
        "Jadwal sambang untuk {} ({}) telah ditetapkan pada {} pukul {}-{} di {}. Silakan konfirmasi kehadiran di aplikasi SiPesantren.",
        detail.santri_name, detail.ticket_code, detail.requested_date, start_hhmm, end_hhmm, spot
    );
    state
        .notifier
        .notify_wali(Some(&detail.id), &detail.wali_phone, "SCHEDULE_SET", &wa_msg)
        .await;
    state.sse.emit_public_flow_event(&detail);

    Ok(detail)
}

pub async fn confirm_schedule_by_wali(
    state: &AppState,
    user: &CurrentUser,
    visit_id: &str,
) -> Result<VisitDetailView, AppError> {
    user.require_role(UserRole::Wali)?;

    let pre_detail = fetch_visit_detail(&state.db, visit_id).await?;
    if pre_detail.wali_id != user.id {
        return Err(AppError::Forbidden);
    }

    let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut tx = state.db.begin().await?;

    let res = sqlx::query(
        "UPDATE visit_requests
         SET current_status = 'CONFIRMED_BY_WALI', updated_at = ?1
         WHERE id = ?2 AND wali_id = ?3 AND current_status = 'SCHEDULED'"
    )
    .bind(&now_iso)
    .bind(visit_id)
    .bind(&user.id)
    .execute(&mut *tx)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::Conflict("Tiket harus berstatus SCHEDULED untuk dikonfirmasi".into()));
    }

    sqlx::query(
        "INSERT INTO visit_status_logs (visit_id, from_status, to_status, actor_id, public_note, created_at)
         VALUES (?1, 'SCHEDULED', 'CONFIRMED_BY_WALI', ?2, 'Wali Santri mengonfirmasi kehadiran sesuai jadwal', ?3)"
    )
    .bind(visit_id)
    .bind(&user.id)
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let detail = fetch_visit_detail(&state.db, visit_id).await?;
    state.sse.emit_public_flow_event(&detail);
    Ok(detail)
}

pub async fn call_santri_to_dormitory(
    state: &AppState,
    user: &CurrentUser,
    visit_id: &str,
) -> Result<VisitDetailView, AppError> {
    user.require_role(UserRole::Admin)?;

    let pre_detail = fetch_visit_detail(&state.db, visit_id).await?;
    let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let note = if pre_detail.has_titipan == 1 {
        "Admin memanggil santri ke asrama beserta informasi ada barang titipan"
    } else {
        "Admin memanggil santri ke asrama (tanpa barang titipan)"
    };

    let mut tx = state.db.begin().await?;

    let res = sqlx::query(
        "UPDATE visit_requests
         SET current_status = 'SANTRI_CALLED', updated_at = ?1
         WHERE id = ?2 AND current_status = 'CONFIRMED_BY_WALI'"
    )
    .bind(&now_iso)
    .bind(visit_id)
    .execute(&mut *tx)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::Conflict(
            "Tiket harus berstatus CONFIRMED_BY_WALI sebelum memanggil santri".into(),
        ));
    }

    sqlx::query(
        "INSERT INTO visit_status_logs (visit_id, from_status, to_status, actor_id, public_note, created_at)
         VALUES (?1, 'CONFIRMED_BY_WALI', 'SANTRI_CALLED', ?2, ?3, ?4)"
    )
    .bind(visit_id)
    .bind(&user.id)
    .bind(note)
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let detail = fetch_visit_detail(&state.db, visit_id).await?;
    state.sse.emit_dormitory_event("dormitory_call", &detail);
    state.sse.emit_public_flow_event(&detail);
    Ok(detail)
}

pub async fn confirm_santri_by_musyrif(
    state: &AppState,
    user: &CurrentUser,
    visit_id: &str,
) -> Result<VisitDetailView, AppError> {
    let pre_detail = fetch_visit_detail(&state.db, visit_id).await?;
    user.require_admin_or_musyrif_for_dorm(&pre_detail.dormitory_id)?;

    let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut tx = state.db.begin().await?;

    let res = sqlx::query(
        "UPDATE visit_requests
         SET current_status = 'SANTRI_CONFIRMED',
             santri_notified_by_musyrif = 1,
             titipan_confirmed_to_wali = 1,
             updated_at = ?1
         WHERE id = ?2 AND current_status = 'SANTRI_CALLED'"
    )
    .bind(&now_iso)
    .bind(visit_id)
    .execute(&mut *tx)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::Conflict("Tiket harus berstatus SANTRI_CALLED".into()));
    }

    sqlx::query(
        "INSERT INTO visit_status_logs (visit_id, from_status, to_status, actor_id, public_note, created_at)
         VALUES (?1, 'SANTRI_CALLED', 'SANTRI_CONFIRMED', ?2, 'Musyrif mengonfirmasi santri & info titipan telah disampaikan; terkonfirmasi ke Wali Santri', ?3)"
    )
    .bind(visit_id)
    .bind(&user.id)
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let detail = fetch_visit_detail(&state.db, visit_id).await?;
    let titipan_info = if detail.has_titipan == 1 {
        "beserta informasi titipan barang "
    } else {
        ""
    };
    let wa_msg = format!(
        "Santri {} telah dikabari oleh Musyrif {} {}dan sedang menuju {}.",
        detail.santri_name,
        detail.dormitory_name,
        titipan_info,
        detail.spot_display()
    );
    state
        .notifier
        .notify_wali(Some(&detail.id), &detail.wali_phone, "SANTRI_READY", &wa_msg)
        .await;
    state.sse.emit_dormitory_event("dormitory_update", &detail);
    state.sse.emit_public_flow_event(&detail);

    Ok(detail)
}

pub async fn cancel_visit(
    state: &AppState,
    user: &CurrentUser,
    visit_id: &str,
    input: CancelVisitInput,
) -> Result<VisitDetailView, AppError> {
    let reason = input.cancel_reason.trim();
    if reason.is_empty() || reason.len() > 250 {
        return Err(AppError::BadRequest(
            "Alasan pembatalan wajib diisi (maks 250 karakter)".into(),
        ));
    }

    let pre_detail = fetch_visit_detail(&state.db, visit_id).await?;
    match user.role {
        UserRole::Wali => {
            if pre_detail.wali_id != user.id {
                return Err(AppError::Forbidden);
            }
            if !matches!(pre_detail.current_status.as_str(), "REQUESTED" | "SCHEDULED") {
                return Err(AppError::Conflict(
                    "Tiket tidak dapat dibatalkan oleh Wali pada tahap ini".into(),
                ));
            }
        }
        UserRole::Admin => {
            if !matches!(
                pre_detail.current_status.as_str(),
                "REQUESTED" | "SCHEDULED" | "CONFIRMED_BY_WALI"
            ) {
                return Err(AppError::Conflict(
                    "Tiket tidak dapat dibatalkan pada tahap ini".into(),
                ));
            }
        }
        _ => return Err(AppError::Forbidden),
    }

    let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let note = format!("Kunjungan dibatalkan: {}", reason);

    let mut tx = state.db.begin().await?;

    let res = sqlx::query(
        "UPDATE visit_requests
         SET current_status = 'CANCELLED', cancel_reason = ?1, updated_at = ?2
         WHERE id = ?3 AND current_status = ?4"
    )
    .bind(reason)
    .bind(&now_iso)
    .bind(visit_id)
    .bind(&pre_detail.current_status)
    .execute(&mut *tx)
    .await?;

    if res.rows_affected() == 0 {
        return Err(AppError::Conflict("Status tiket telah berubah".into()));
    }

    sqlx::query(
        "INSERT INTO visit_status_logs (visit_id, from_status, to_status, actor_id, public_note, created_at)
         VALUES (?1, ?2, 'CANCELLED', ?3, ?4, ?5)"
    )
    .bind(visit_id)
    .bind(&pre_detail.current_status)
    .bind(&user.id)
    .bind(&note)
    .bind(&now_iso)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let detail = fetch_visit_detail(&state.db, visit_id).await?;
    state.sse.emit_public_flow_event(&detail);
    Ok(detail)
}
