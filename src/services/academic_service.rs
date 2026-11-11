use chrono::Utc;
use serde::Deserialize;

use crate::{
    auth::CurrentUser,
    error::AppError,
    models::{
        compute_grade_predicate, validate_iso_date, ExamGradeView, ExamView,
        RaporSubjectRow, Subject,
    },
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateExamInput {
    pub term_id: String,
    pub subject_id: String,
    pub exam_type: String,
    pub class_level: String,
    pub exam_date: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertExamGradeInput {
    pub exam_id: String,
    pub santri_id: String,
    pub score: f64,
    pub teacher_note: Option<String>,
}

pub async fn create_exam_session(
    state: &AppState,
    actor: &CurrentUser,
    input: CreateExamInput,
) -> Result<ExamView, AppError> {
    actor.require_admin_or_asatidz()?;
    let clean_date = validate_iso_date(&input.exam_date)?;

    if !matches!(
        input.exam_type.as_str(),
        "HARIAN" | "UTS" | "UAS" | "SYAFAHI_LISAN"
    ) {
        return Err(AppError::BadRequest("Jenis ujian tidak valid".into()));
    }
    if input.title.trim().is_empty() || input.class_level.trim().is_empty() {
        return Err(AppError::BadRequest("Judul ujian dan tingkat kelas wajib diisi".into()));
    }

    let id = uuid::Uuid::now_v7().to_string();
    sqlx::query(
        "INSERT INTO exams (id, term_id, subject_id, exam_type, class_level, exam_date, examiner_id, title)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )
    .bind(&id)
    .bind(&input.term_id)
    .bind(&input.subject_id)
    .bind(&input.exam_type)
    .bind(input.class_level.trim())
    .bind(&clean_date)
    .bind(&actor.id)
    .bind(input.title.trim())
    .execute(&state.db)
    .await?;

    let view: ExamView = sqlx::query_as(
        "SELECT e.id, e.term_id, t.term_name, e.subject_id, sub.code AS subject_code,
                sub.name AS subject_name, sub.category AS subject_category, sub.kkm_score,
                e.exam_type, e.class_level, e.exam_date, e.examiner_id,
                u.full_name AS examiner_name, e.title,
                (SELECT COUNT(*) FROM exam_grades g WHERE g.exam_id = e.id) AS graded_count,
                e.created_at
         FROM exams e
         JOIN academic_terms t ON t.id = e.term_id
         JOIN subjects sub ON sub.id = e.subject_id
         JOIN users u ON u.id = e.examiner_id
         WHERE e.id = ?1"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await?;

    Ok(view)
}

pub async fn upsert_exam_grade(
    state: &AppState,
    actor: &CurrentUser,
    input: UpsertExamGradeInput,
) -> Result<ExamGradeView, AppError> {
    actor.require_admin_or_asatidz()?;

    if input.score.is_nan() || !(0.0..=100.0).contains(&input.score) {
        return Err(AppError::BadRequest(
            "Nilai ujian harus berada pada rentang 0.0 hingga 100.0".into(),
        ));
    }

    let wali_phone: Option<String> = sqlx::query_scalar(
        "SELECT u.phone_number
         FROM santri s
         JOIN users u ON u.id = s.primary_wali_id
         WHERE s.id = ?1 AND s.is_active = 1"
    )
    .bind(&input.santri_id)
    .fetch_optional(&state.db)
    .await?;

    let wali_phone = wali_phone.ok_or_else(|| AppError::NotFound("Santri tidak ditemukan".into()))?;

    let (letter_grade, predicate_label) = compute_grade_predicate(input.score);
    let now_iso = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let id = uuid::Uuid::now_v7().to_string();

    sqlx::query(
        "INSERT INTO exam_grades (
            id, exam_id, santri_id, score, letter_grade, predicate_label,
            teacher_note, recorded_by_id, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(exam_id, santri_id) DO UPDATE SET
            score = excluded.score,
            letter_grade = excluded.letter_grade,
            predicate_label = excluded.predicate_label,
            teacher_note = excluded.teacher_note,
            recorded_by_id = excluded.recorded_by_id,
            updated_at = excluded.updated_at"
    )
    .bind(&id)
    .bind(&input.exam_id)
    .bind(&input.santri_id)
    .bind(input.score)
    .bind(letter_grade)
    .bind(predicate_label)
    .bind(input.teacher_note.as_deref().map(str::trim))
    .bind(&actor.id)
    .bind(&now_iso)
    .execute(&state.db)
    .await?;

    let view: ExamGradeView = sqlx::query_as(
        "SELECT g.id, g.exam_id, e.title AS exam_title, e.exam_type, e.exam_date,
                e.term_id, t.term_name, e.subject_id, sub.code AS subject_code,
                sub.name AS subject_name, sub.category AS subject_category, sub.kkm_score,
                g.santri_id, s.full_name AS santri_name, s.nis AS santri_nis,
                g.score, g.letter_grade, g.predicate_label, g.teacher_note,
                u.full_name AS recorded_by_name, g.updated_at
         FROM exam_grades g
         JOIN exams e ON e.id = g.exam_id
         JOIN academic_terms t ON t.id = e.term_id
         JOIN subjects sub ON sub.id = e.subject_id
         JOIN santri s ON s.id = g.santri_id
         JOIN users u ON u.id = g.recorded_by_id
         WHERE g.exam_id = ?1 AND g.santri_id = ?2"
    )
    .bind(&input.exam_id)
    .bind(&input.santri_id)
    .fetch_one(&state.db)
    .await?;

    let wa_msg = format!(
        "Hasil Evaluasi Akademik {}: {} ({}) memperoleh nilai {:.1} [{} - {}].",
        view.santri_name,
        view.subject_name,
        view.exam_type,
        view.score,
        view.letter_grade,
        view.predicate_label
    );
    state
        .notifier
        .notify_wali(None, &wali_phone, "RAPOR_PUBLISHED", &wa_msg)
        .await;

    Ok(view)
}

fn format_opt_score(opt: Option<f64>) -> String {
    match opt {
        Some(v) => format!("{:.1}", v),
        None => "-".to_string(),
    }
}

/// Menghasilkan rekap tabel Rapor Digital per santri untuk semester tertentu
pub async fn build_santri_rapor_rows(
    db: &sqlx::SqlitePool,
    santri_id: &str,
    term_id: &str,
) -> Result<Vec<RaporSubjectRow>, AppError> {
    let subjects: Vec<Subject> = sqlx::query_as(
        "SELECT id, code, name, category, kkm_score, is_active, created_at
         FROM subjects
         WHERE is_active = 1
         ORDER BY category ASC, code ASC"
    )
    .fetch_all(db)
    .await?;

    let grades: Vec<ExamGradeView> = sqlx::query_as(
        "SELECT g.id, g.exam_id, e.title AS exam_title, e.exam_type, e.exam_date,
                e.term_id, t.term_name, e.subject_id, sub.code AS subject_code,
                sub.name AS subject_name, sub.category AS subject_category, sub.kkm_score,
                g.santri_id, s.full_name AS santri_name, s.nis AS santri_nis,
                g.score, g.letter_grade, g.predicate_label, g.teacher_note,
                u.full_name AS recorded_by_name, g.updated_at
         FROM exam_grades g
         JOIN exams e ON e.id = g.exam_id
         JOIN academic_terms t ON t.id = e.term_id
         JOIN subjects sub ON sub.id = e.subject_id
         JOIN santri s ON s.id = g.santri_id
         JOIN users u ON u.id = g.recorded_by_id
         WHERE g.santri_id = ?1 AND e.term_id = ?2
         ORDER BY e.exam_date DESC"
    )
    .bind(santri_id)
    .bind(term_id)
    .fetch_all(db)
    .await?;

    let mut rows = Vec::with_capacity(subjects.len());
    for sub in subjects {
        let sub_grades: Vec<&ExamGradeView> = grades
            .iter()
            .filter(|g| g.subject_id == sub.id)
            .collect();

        let find_avg = |etype: &str| -> Option<f64> {
            let vals: Vec<f64> = sub_grades
                .iter()
                .filter(|g| g.exam_type == etype)
                .map(|g| g.score)
                .collect();
            if vals.is_empty() {
                None
            } else {
                let sum: f64 = vals.iter().sum();
                Some(((sum / vals.len() as f64) * 10.0).round() / 10.0)
            }
        };

        let harian_score = find_avg("HARIAN");
        let uts_score = find_avg("UTS");
        let uas_score = find_avg("UAS");
        let syafahi_score = find_avg("SYAFAHI_LISAN");

        let present_components: Vec<f64> = [harian_score, uts_score, uas_score, syafahi_score]
            .into_iter()
            .flatten()
            .collect();

        let final_score = if present_components.is_empty() {
            0.0
        } else {
            let sum: f64 = present_components.iter().sum();
            ((sum / present_components.len() as f64) * 10.0).round() / 10.0
        };

        let (letter, pred, final_display) = if present_components.is_empty() {
            ("-", "Belum Dinilai", "-".to_string())
        } else {
            let (l, p) = compute_grade_predicate(final_score);
            (l, p, format!("{:.1}", final_score))
        };

        rows.push(RaporSubjectRow {
            subject_code: sub.code,
            subject_name: sub.name,
            category: sub.category,
            kkm_score: sub.kkm_score,
            harian_score,
            uts_score,
            uas_score,
            syafahi_score,
            harian_display: format_opt_score(harian_score),
            uts_display: format_opt_score(uts_score),
            uas_display: format_opt_score(uas_score),
            syafahi_display: format_opt_score(syafahi_score),
            final_score,
            final_display,
            letter_grade: letter.to_string(),
            predicate_label: pred.to_string(),
            passed_kkm: !present_components.is_empty() && final_score >= sub.kkm_score as f64,
        });
    }

    Ok(rows)
}
