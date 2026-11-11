use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Wali,
    Musyrif,
    Asatidz,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "ADMIN",
            Self::Wali => "WALI",
            Self::Musyrif => "MUSYRIF",
            Self::Asatidz => "ASATIDZ",
        }
    }

    pub fn from_db(val: &str) -> Option<Self> {
        match val {
            "ADMIN" => Some(Self::Admin),
            "WALI" => Some(Self::Wali),
            "MUSYRIF" => Some(Self::Musyrif),
            "ASATIDZ" => Some(Self::Asatidz),
            _ => None,
        }
    }
}

/// Memvalidasi dan menormalisasi tanggal ke format kanonis YYYY-MM-DD
pub fn validate_iso_date(raw: &str) -> Result<String, AppError> {
    let d = NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest(format!("Format tanggal '{}' tidak valid (wajib YYYY-MM-DD)", raw)))?;
    Ok(d.format("%Y-%m-%d").to_string())
}

/// Memvalidasi dan menormalisasi jam ke format kanonis HH:MM (zero-padded, misal "9:00" -> "09:00")
pub fn validate_hhmm_time(raw: &str) -> Result<String, AppError> {
    let t = NaiveTime::parse_from_str(raw.trim(), "%H:%M")
        .map_err(|_| AppError::BadRequest(format!("Format jam '{}' tidak valid (wajib HH:MM)", raw)))?;
    Ok(t.format("%H:%M").to_string())
}

pub fn mask_santri_name(full_name: &str) -> String {
    let parts: Vec<&str> = full_name.split_whitespace().collect();
    match parts.as_slice() {
        [] => "Santri".to_string(),
        [single] => (*single).to_string(),
        [first, second, ..] => {
            let initial = second.chars().next().unwrap_or(' ');
            format!("{} {}.", first, initial)
        }
    }
}

pub fn compute_grade_predicate(score: f64) -> (&'static str, &'static str) {
    if score >= 90.0 {
        ("A", "Mumtaz (Istimewa)")
    } else if score >= 80.0 {
        ("B", "Jayyid Jiddan (Sangat Baik)")
    } else if score >= 70.0 {
        ("C", "Jayyid (Baik)")
    } else if score >= 60.0 {
        ("D", "Maqbul (Cukup)")
    } else {
        ("E", "Dhaif (Perlu Pembinaan)")
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: String,
    pub full_name: String,
    pub phone_number: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub dormitory_id: Option<String>,
    pub is_active: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Dormitory {
    pub id: String,
    pub name: String,
    pub gender_zone: String,
    pub building_location: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Santri {
    pub id: String,
    pub nis: String,
    pub full_name: String,
    pub class_level: String,
    pub dormitory_id: String,
    pub room_name: String,
    pub primary_wali_id: String,
    pub total_juz_memorized: i64,
    pub current_surah_focus: String,
    pub is_active: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct VisitDetailView {
    pub id: String,
    pub ticket_code: String,
    pub wali_id: String,
    pub wali_name: String,
    pub wali_phone: String,
    pub santri_id: String,
    pub santri_name: String,
    pub santri_nis: String,
    pub dormitory_id: String,
    pub dormitory_name: String,
    pub room_name: String,
    pub requested_date: String,
    pub requested_session: String,
    pub scheduled_start: Option<String>,
    pub scheduled_end: Option<String>,
    pub meeting_spot: Option<String>,
    pub visitor_count: i64,
    pub has_titipan: i64,
    pub titipan_description: Option<String>,
    pub current_status: String,
    pub santri_notified_by_musyrif: i64,
    pub titipan_confirmed_to_wali: i64,
    pub cancel_reason: Option<String>,
    pub has_logbook: i64,
    pub meeting_timestamp: Option<String>,
    pub meeting_notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl VisitDetailView {
    pub fn schedule_window_display(&self) -> String {
        match (&self.scheduled_start, &self.scheduled_end) {
            (Some(s), Some(e)) => format!("{} - {}", s, e),
            _ => "Menunggu Penjadwalan".to_string(),
        }
    }

    pub fn spot_display(&self) -> &str {
        self.meeting_spot.as_deref().unwrap_or("Belum Ditentukan")
    }

    pub fn titipan_display(&self) -> &str {
        if self.has_titipan == 1 {
            self.titipan_description.as_deref().unwrap_or("Ada Titipan")
        } else {
            "Tanpa Titipan"
        }
    }

    pub fn meeting_notes_display(&self) -> &str {
        self.meeting_notes.as_deref().unwrap_or("-")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicFlowItemView {
    pub ticket_code: String,
    pub santri_masked_name: String,
    pub dormitory_name: String,
    pub requested_date: String,
    pub requested_session: String,
    pub scheduled_window_display: String,
    pub meeting_spot_display: String,
    pub has_titipan: bool,
    pub titipan_confirmed_to_wali: bool,
    pub current_status: String,
    pub has_private_logbook_photo: bool,
    pub updated_at: String,
}

impl From<&VisitDetailView> for PublicFlowItemView {
    fn from(v: &VisitDetailView) -> Self {
        Self {
            ticket_code: v.ticket_code.clone(),
            santri_masked_name: mask_santri_name(&v.santri_name),
            dormitory_name: v.dormitory_name.clone(),
            requested_date: v.requested_date.clone(),
            requested_session: v.requested_session.clone(),
            scheduled_window_display: v.schedule_window_display(),
            meeting_spot_display: v.spot_display().to_string(),
            has_titipan: v.has_titipan == 1,
            titipan_confirmed_to_wali: v.titipan_confirmed_to_wali == 1,
            current_status: v.current_status.clone(),
            has_private_logbook_photo: v.has_logbook == 1,
            updated_at: v.updated_at.clone(),
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct VisitStatusLog {
    pub id: i64,
    pub visit_id: String,
    pub from_status: Option<String>,
    pub to_status: String,
    pub actor_id: Option<String>,
    pub public_note: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TahfidzLogView {
    pub id: String,
    pub santri_id: String,
    pub santri_name: String,
    pub santri_nis: String,
    pub evaluator_id: String,
    pub evaluator_name: String,
    pub session_date: String,
    pub log_type: String,
    pub juz_number: i64,
    pub surah_or_kitab: String,
    pub ayah_or_bait_start: i64,
    pub ayah_or_bait_end: i64,
    pub predicate: String,
    pub score_numeric: i64,
    pub notes: Option<String>,
    pub created_at: String,
}

impl TahfidzLogView {
    pub fn notes_display(&self) -> &str {
        self.notes.as_deref().unwrap_or("-")
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AcademicTerm {
    pub id: String,
    pub term_name: String,
    pub academic_year: String,
    pub semester_label: String,
    pub is_active: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Subject {
    pub id: String,
    pub code: String,
    pub name: String,
    pub category: String,
    pub kkm_score: i64,
    pub is_active: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ExamView {
    pub id: String,
    pub term_id: String,
    pub term_name: String,
    pub subject_id: String,
    pub subject_code: String,
    pub subject_name: String,
    pub subject_category: String,
    pub kkm_score: i64,
    pub exam_type: String,
    pub class_level: String,
    pub exam_date: String,
    pub examiner_id: String,
    pub examiner_name: String,
    pub title: String,
    pub graded_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ExamGradeView {
    pub id: String,
    pub exam_id: String,
    pub exam_title: String,
    pub exam_type: String,
    pub exam_date: String,
    pub term_id: String,
    pub term_name: String,
    pub subject_id: String,
    pub subject_code: String,
    pub subject_name: String,
    pub subject_category: String,
    pub kkm_score: i64,
    pub santri_id: String,
    pub santri_name: String,
    pub santri_nis: String,
    pub score: f64,
    pub letter_grade: String,
    pub predicate_label: String,
    pub teacher_note: Option<String>,
    pub recorded_by_name: String,
    pub updated_at: String,
}

impl ExamGradeView {
    pub fn teacher_note_display(&self) -> &str {
        self.teacher_note.as_deref().unwrap_or("-")
    }
}

/// Baris agregasi per Mata Pelajaran untuk Rapor Digital Santri (Askama Display-safe)
#[derive(Debug, Clone, Serialize)]
pub struct RaporSubjectRow {
    pub subject_code: String,
    pub subject_name: String,
    pub category: String,
    pub kkm_score: i64,
    pub harian_score: Option<f64>,
    pub uts_score: Option<f64>,
    pub uas_score: Option<f64>,
    pub syafahi_score: Option<f64>,
    pub harian_display: String,
    pub uts_display: String,
    pub uas_display: String,
    pub syafahi_display: String,
    pub final_score: f64,
    pub final_display: String,
    pub letter_grade: String,
    pub predicate_label: String,
    pub passed_kkm: bool,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AttendanceLogView {
    pub id: String,
    pub santri_id: String,
    pub santri_name: String,
    pub attendance_date: String,
    pub activity_type: String,
    pub status: String,
    pub note: Option<String>,
    pub recorded_by_name: String,
    pub created_at: String,
}

impl AttendanceLogView {
    pub fn note_display(&self) -> &str {
        self.note.as_deref().unwrap_or("-")
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct SantriNoteView {
    pub id: String,
    pub santri_id: String,
    pub santri_name: String,
    pub note_date: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub recorded_by_name: String,
    pub created_at: String,
}
