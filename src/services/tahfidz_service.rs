use serde::Deserialize;
use crate::{
    auth::CurrentUser,
    error::AppError,
    models::{validate_iso_date, TahfidzLogView},
    AppState,
};

pub const SELECT_TAHFIDZ_LOG_SQL: &str = r#"
SELECT
    t.id,
    t.santri_id,
    s.full_name AS santri_name,
    s.nis AS santri_nis,
    t.evaluator_id,
    u.full_name AS evaluator_name,
    t.session_date,
    t.log_type,
    t.juz_number,
    t.surah_or_kitab,
    t.ayah_or_bait_start,
    t.ayah_or_bait_end,
    t.predicate,
    t.score_numeric,
    t.notes,
    t.created_at
FROM tahfidz_logs t
JOIN santri s ON s.id = t.santri_id
JOIN users u ON u.id = t.evaluator_id
"#;

pub fn deserialize_optional_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OptIntOrStr {
        Int(i64),
        Str(String),
    }

    match Option::<OptIntOrStr>::deserialize(deserializer)? {
        None => Ok(None),
        Some(OptIntOrStr::Int(v)) => Ok(Some(v)),
        Some(OptIntOrStr::Str(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                trimmed.parse::<i64>().map(Some).map_err(serde::de::Error::custom)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTahfidzInput {
    pub santri_id: String,
    pub session_date: String,
    pub log_type: String,
    pub juz_number: i64,
    pub surah_or_kitab: String,
    pub ayah_or_bait_start: i64,
    pub ayah_or_bait_end: i64,
    pub predicate: String,
    pub score_numeric: i64,
    pub notes: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_i64")]
    pub update_total_juz: Option<i64>,
}

pub async fn record_tahfidz_log(
    state: &AppState,
    actor: &CurrentUser,
    input: CreateTahfidzInput,
) -> Result<TahfidzLogView, AppError> {
    let clean_date = validate_iso_date(&input.session_date)?;

    let santri_row: Option<(String, String, String)> = sqlx::query_as(
        "SELECT s.dormitory_id, s.full_name, u.phone_number
         FROM santri s
         JOIN users u ON u.id = s.primary_wali_id
         WHERE s.id = ?1 AND s.is_active = 1"
    )
    .bind(&input.santri_id)
    .fetch_optional(&state.db)
    .await?;

    let (dorm_id, santri_name, wali_phone) =
        santri_row.ok_or_else(|| AppError::NotFound("Santri tidak ditemukan".into()))?;

    actor.require_staff_for_dorm(&dorm_id)?;

    if !matches!(
        input.log_type.as_str(),
        "ZIYADAH" | "MUROJAAH" | "TASMI_JUZ" | "MUTUN_KITAB"
    ) {
        return Err(AppError::BadRequest("Jenis setoran tidak valid".into()));
    }
    if !(1..=30).contains(&input.juz_number) {
        return Err(AppError::BadRequest("Nomor Juz harus antara 1 sampai 30".into()));
    }
    if input.surah_or_kitab.trim().is_empty() {
        return Err(AppError::BadRequest("Nama Surah atau Kitab wajib diisi".into()));
    }
    if input.ayah_or_bait_start < 1 || input.ayah_or_bait_end < input.ayah_or_bait_start {
        return Err(AppError::BadRequest("Rentang ayat/bait tidak valid".into()));
    }
    if !matches!(
        input.predicate.as_str(),
        "MUMTAZ" | "JAYYID_JIDDAN" | "JAYYID" | "MAQBUL" | "MENGULANG"
    ) {
        return Err(AppError::BadRequest("Predikat kelancaran tidak valid".into()));
    }
    if !(0..=100).contains(&input.score_numeric) {
        return Err(AppError::BadRequest("Skor harus antara 0 sampai 100".into()));
    }

    let id = uuid::Uuid::now_v7().to_string();
    let mut tx = state.db.begin().await?;

    sqlx::query(
        "INSERT INTO tahfidz_logs (
            id, santri_id, evaluator_id, session_date, log_type,
            juz_number, surah_or_kitab, ayah_or_bait_start, ayah_or_bait_end,
            predicate, score_numeric, notes
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"
    )
    .bind(&id)
    .bind(&input.santri_id)
    .bind(&actor.id)
    .bind(&clean_date)
    .bind(&input.log_type)
    .bind(input.juz_number)
    .bind(input.surah_or_kitab.trim())
    .bind(input.ayah_or_bait_start)
    .bind(input.ayah_or_bait_end)
    .bind(&input.predicate)
    .bind(input.score_numeric)
    .bind(input.notes.as_deref().map(str::trim))
    .execute(&mut *tx)
    .await?;

    if input.log_type == "ZIYADAH" || input.log_type == "TASMI_JUZ" {
        let focus = format!(
            "Juz {} ({} : {}-{})",
            input.juz_number,
            input.surah_or_kitab.trim(),
            input.ayah_or_bait_start,
            input.ayah_or_bait_end
        );
        if let Some(new_total) = input.update_total_juz {
            if (0..=30).contains(&new_total) {
                sqlx::query(
                    "UPDATE santri SET total_juz_memorized = ?1, current_surah_focus = ?2 WHERE id = ?3"
                )
                .bind(new_total)
                .bind(&focus)
                .bind(&input.santri_id)
                .execute(&mut *tx)
                .await?;
            }
        } else {
            sqlx::query("UPDATE santri SET current_surah_focus = ?1 WHERE id = ?2")
                .bind(&focus)
                .bind(&input.santri_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    tx.commit().await?;

    let sql = format!("{} WHERE t.id = ?1", SELECT_TAHFIDZ_LOG_SQL);
    let view: TahfidzLogView = sqlx::query_as(&sql)
        .bind(&id)
        .fetch_one(&state.db)
        .await?;

    let wa_msg = format!(
        "Laporan Tahfidz {}: {} ({} ayat/bait {}-{}) dengan predikat {} (Skor {}). Penyimak: {}.",
        santri_name,
        view.log_type,
        view.surah_or_kitab,
        view.ayah_or_bait_start,
        view.ayah_or_bait_end,
        view.predicate,
        view.score_numeric,
        view.evaluator_name
    );
    state
        .notifier
        .notify_wali(None, &wali_phone, "TAHFIDZ_RECORDED", &wa_msg)
        .await;

    Ok(view)
}
