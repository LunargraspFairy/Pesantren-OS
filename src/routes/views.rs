use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
};
use chrono::Utc;
use serde::Deserialize;

use crate::{
    auth::CurrentUser,
    config::PONDOK_LOGO_URL,
    error::{AppError, HtmlTemplate},
    models::{
        AcademicTerm, AttendanceLogView, Dormitory, ExamGradeView, ExamView,
        PublicFlowItemView, RaporSubjectRow, Santri, SantriNoteView, Subject,
        TahfidzLogView, UserRole, VisitDetailView, VisitStatusLog,
    },
    services::{
        academic_service::build_santri_rapor_rows,
        tahfidz_service::SELECT_TAHFIDZ_LOG_SQL,
        visit_service::SELECT_VISIT_DETAIL_SQL,
    },
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct DateFilterQuery {
    pub date: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WaliFilterQuery {
    pub santri_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DormBoardQuery {
    pub dorm_id: Option<String>,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub logo_url: &'static str,
    pub error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "public_flow.html")]
pub struct PublicFlowTemplate {
    pub logo_url: &'static str,
    pub filter_date: String,
    pub items: Vec<PublicFlowItemView>,
    pub selected_ticket: Option<PublicFlowItemView>,
    pub selected_logs: Vec<VisitStatusLog>,
}

#[derive(Template)]
#[template(path = "wali_dashboard.html")]
pub struct WaliDashboardTemplate {
    pub logo_url: &'static str,
    pub user_name: String,
    pub santri_list: Vec<Santri>,
    pub active_santri: Santri,
    pub active_term: AcademicTerm,
    pub active_visits: Vec<VisitDetailView>,
    pub completed_logbooks: Vec<VisitDetailView>,
    pub tahfidz_logs: Vec<TahfidzLogView>,
    pub rapor_rows: Vec<RaporSubjectRow>,
    pub exam_grades: Vec<ExamGradeView>,
    pub attendance_logs: Vec<AttendanceLogView>,
    pub santri_notes: Vec<SantriNoteView>,
    pub hadir_count: usize,
    pub sakit_izin_count: usize,
}

#[derive(Template)]
#[template(path = "asatidz_dashboard.html")]
pub struct AsatidzDashboardTemplate {
    pub logo_url: &'static str,
    pub user_name: String,
    pub today_date: String,
    pub active_term: AcademicTerm,
    pub santri_list: Vec<Santri>,
    pub subjects: Vec<Subject>,
    pub exams: Vec<ExamView>,
    pub recent_tahfidz: Vec<TahfidzLogView>,
    pub recent_grades: Vec<ExamGradeView>,
}

#[derive(Template)]
#[template(path = "admin_dashboard.html")]
pub struct AdminDashboardTemplate {
    pub logo_url: &'static str,
    pub user_name: String,
    pub filter_date: String,
    pub filter_status: String,
    pub visits: Vec<VisitDetailView>,
    pub santri_count: i64,
    pub tahfidz_total_count: i64,
    pub exam_total_count: i64,
}

#[derive(Template)]
#[template(path = "musyrif_board.html")]
pub struct MusyrifBoardTemplate {
    pub logo_url: &'static str,
    pub user_name: String,
    pub today_date: String,
    pub is_admin: bool,
    pub current_dorm: Dormitory,
    pub all_dorms: Vec<Dormitory>,
    pub dorm_santri: Vec<Santri>,
    pub called_visits: Vec<VisitDetailView>,
    pub confirmed_visits: Vec<VisitDetailView>,
    pub recent_attendance: Vec<AttendanceLogView>,
    pub recent_notes: Vec<SantriNoteView>,
}

async fn fetch_active_term(db: &sqlx::SqlitePool) -> Result<AcademicTerm, AppError> {
    sqlx::query_as::<_, AcademicTerm>(
        "SELECT id, term_name, academic_year, semester_label, is_active, created_at
         FROM academic_terms
         WHERE is_active = 1
         ORDER BY created_at DESC LIMIT 1"
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound("Periode semester aktif belum diatur".into()))
}

pub async fn root_redirect(user: Option<CurrentUser>) -> Redirect {
    match user {
        Some(u) => Redirect::to(u.home_redirect_path()),
        None => Redirect::to("/alur"),
    }
}

pub async fn login_page(user: Option<CurrentUser>) -> impl IntoResponse {
    if let Some(u) = user {
        return Redirect::to(u.home_redirect_path()).into_response();
    }
    HtmlTemplate(LoginTemplate {
        logo_url: PONDOK_LOGO_URL,
        error_message: None,
    })
    .into_response()
}

pub async fn public_flow_page(
    State(state): State<AppState>,
    Query(query): Query<DateFilterQuery>,
) -> Result<HtmlTemplate<PublicFlowTemplate>, AppError> {
    let filter_date = query
        .date
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

    let sql = format!(
        "{} WHERE v.requested_date = ?1 ORDER BY v.updated_at DESC",
        SELECT_VISIT_DETAIL_SQL
    );
    let rows: Vec<VisitDetailView> = sqlx::query_as(&sql)
        .bind(&filter_date)
        .fetch_all(&state.db)
        .await?;

    let items = rows.iter().map(PublicFlowItemView::from).collect();

    Ok(HtmlTemplate(PublicFlowTemplate {
        logo_url: PONDOK_LOGO_URL,
        filter_date,
        items,
        selected_ticket: None,
        selected_logs: Vec::new(),
    }))
}

pub async fn public_flow_ticket_page(
    State(state): State<AppState>,
    Path(ticket_code): Path<String>,
) -> Result<HtmlTemplate<PublicFlowTemplate>, AppError> {
    let sql = format!("{} WHERE v.ticket_code = ?1", SELECT_VISIT_DETAIL_SQL);
    let detail: VisitDetailView = sqlx::query_as(&sql)
        .bind(&ticket_code)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Kode tiket tidak ditemukan".into()))?;

    let selected_ticket = PublicFlowItemView::from(&detail);
    let selected_logs: Vec<VisitStatusLog> = sqlx::query_as(
        "SELECT id, visit_id, from_status, to_status, actor_id, public_note, created_at
         FROM visit_status_logs
         WHERE visit_id = ?1
         ORDER BY id ASC"
    )
    .bind(&detail.id)
    .fetch_all(&state.db)
    .await?;

    let list_sql = format!(
        "{} WHERE v.requested_date = ?1 ORDER BY v.updated_at DESC",
        SELECT_VISIT_DETAIL_SQL
    );
    let rows: Vec<VisitDetailView> = sqlx::query_as(&list_sql)
        .bind(&detail.requested_date)
        .fetch_all(&state.db)
        .await?;
    let items = rows.iter().map(PublicFlowItemView::from).collect();

    Ok(HtmlTemplate(PublicFlowTemplate {
        logo_url: PONDOK_LOGO_URL,
        filter_date: detail.requested_date,
        items,
        selected_ticket: Some(selected_ticket),
        selected_logs,
    }))
}

pub async fn wali_dashboard_page(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(query): Query<WaliFilterQuery>,
) -> Result<HtmlTemplate<WaliDashboardTemplate>, AppError> {
    user.require_role(UserRole::Wali)?;

    let santri_list: Vec<Santri> = sqlx::query_as(
        "SELECT id, nis, full_name, class_level, dormitory_id, room_name, primary_wali_id,
                total_juz_memorized, current_surah_focus, is_active, created_at
         FROM santri
         WHERE primary_wali_id = ?1 AND is_active = 1
         ORDER BY full_name ASC"
    )
    .bind(&user.id)
    .fetch_all(&state.db)
    .await?;

    let active_santri = if let Some(ref req_id) = query.santri_id {
        santri_list
            .iter()
            .find(|s| &s.id == req_id)
            .cloned()
            .ok_or(AppError::Forbidden)?
    } else {
        santri_list
            .first()
            .cloned()
            .ok_or_else(|| AppError::NotFound("Belum ada data santri terdaftar".into()))?
    };

    let active_term = fetch_active_term(&state.db).await?;

    let visit_sql = format!(
        "{} WHERE v.wali_id = ?1 AND v.santri_id = ?2 ORDER BY v.created_at DESC",
        SELECT_VISIT_DETAIL_SQL
    );
    let all_visits: Vec<VisitDetailView> = sqlx::query_as(&visit_sql)
        .bind(&user.id)
        .bind(&active_santri.id)
        .fetch_all(&state.db)
        .await?;

    let (completed_logbooks, active_visits): (Vec<_>, Vec<_>) = all_visits
        .into_iter()
        .partition(|v| v.current_status == "COMPLETED");

    let tahfidz_sql = format!(
        "{} WHERE t.santri_id = ?1 ORDER BY t.session_date DESC, t.created_at DESC LIMIT 50",
        SELECT_TAHFIDZ_LOG_SQL
    );
    let tahfidz_logs: Vec<TahfidzLogView> = sqlx::query_as(&tahfidz_sql)
        .bind(&active_santri.id)
        .fetch_all(&state.db)
        .await?;

    let rapor_rows = build_santri_rapor_rows(&state.db, &active_santri.id, &active_term.id).await?;

    let exam_grades: Vec<ExamGradeView> = sqlx::query_as(
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
         WHERE g.santri_id = ?1
         ORDER BY e.exam_date DESC LIMIT 50"
    )
    .bind(&active_santri.id)
    .fetch_all(&state.db)
    .await?;

    let attendance_logs: Vec<AttendanceLogView> = sqlx::query_as(
        "SELECT a.id, a.santri_id, s.full_name AS santri_name, a.attendance_date,
                a.activity_type, a.status, a.note, u.full_name AS recorded_by_name, a.created_at
         FROM attendance_logs a
         JOIN santri s ON s.id = a.santri_id
         JOIN users u ON u.id = a.recorded_by_id
         WHERE a.santri_id = ?1
         ORDER BY a.attendance_date DESC LIMIT 60"
    )
    .bind(&active_santri.id)
    .fetch_all(&state.db)
    .await?;

    let hadir_count = attendance_logs.iter().filter(|a| a.status == "HADIR").count();
    let sakit_izin_count = attendance_logs
        .iter()
        .filter(|a| a.status == "SAKIT" || a.status == "IZIN")
        .count();

    let santri_notes: Vec<SantriNoteView> = sqlx::query_as(
        "SELECT n.id, n.santri_id, s.full_name AS santri_name, n.note_date,
                n.category, n.title, n.description, u.full_name AS recorded_by_name, n.created_at
         FROM santri_notes n
         JOIN santri s ON s.id = n.santri_id
         JOIN users u ON u.id = n.recorded_by_id
         WHERE n.santri_id = ?1
         ORDER BY n.note_date DESC LIMIT 30"
    )
    .bind(&active_santri.id)
    .fetch_all(&state.db)
    .await?;

    Ok(HtmlTemplate(WaliDashboardTemplate {
        logo_url: PONDOK_LOGO_URL,
        user_name: user.full_name,
        santri_list,
        active_santri,
        active_term,
        active_visits,
        completed_logbooks,
        tahfidz_logs,
        rapor_rows,
        exam_grades,
        attendance_logs,
        santri_notes,
        hadir_count,
        sakit_izin_count,
    }))
}

pub async fn asatidz_dashboard_page(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<HtmlTemplate<AsatidzDashboardTemplate>, AppError> {
    user.require_admin_or_asatidz()?;

    let today_date = Utc::now().format("%Y-%m-%d").to_string();
    let active_term = fetch_active_term(&state.db).await?;

    let santri_list: Vec<Santri> = sqlx::query_as(
        "SELECT id, nis, full_name, class_level, dormitory_id, room_name, primary_wali_id,
                total_juz_memorized, current_surah_focus, is_active, created_at
         FROM santri WHERE is_active = 1 ORDER BY full_name ASC"
    )
    .fetch_all(&state.db)
    .await?;

    let subjects: Vec<Subject> = sqlx::query_as(
        "SELECT id, code, name, category, kkm_score, is_active, created_at
         FROM subjects WHERE is_active = 1 ORDER BY category ASC, code ASC"
    )
    .fetch_all(&state.db)
    .await?;

    let exams: Vec<ExamView> = sqlx::query_as(
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
         ORDER BY e.exam_date DESC LIMIT 30"
    )
    .fetch_all(&state.db)
    .await?;

    let tahfidz_sql = format!(
        "{} ORDER BY t.session_date DESC, t.created_at DESC LIMIT 25",
        SELECT_TAHFIDZ_LOG_SQL
    );
    let recent_tahfidz: Vec<TahfidzLogView> = sqlx::query_as(&tahfidz_sql)
        .fetch_all(&state.db)
        .await?;

    let recent_grades: Vec<ExamGradeView> = sqlx::query_as(
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
         ORDER BY g.updated_at DESC LIMIT 25"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(HtmlTemplate(AsatidzDashboardTemplate {
        logo_url: PONDOK_LOGO_URL,
        user_name: user.full_name,
        today_date,
        active_term,
        santri_list,
        subjects,
        exams,
        recent_tahfidz,
        recent_grades,
    }))
}

pub async fn admin_dashboard_page(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(query): Query<DateFilterQuery>,
) -> Result<HtmlTemplate<AdminDashboardTemplate>, AppError> {
    user.require_role(UserRole::Admin)?;

    let filter_date = query
        .date
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());
    let filter_status = query.status.unwrap_or_else(|| "ALL".to_string());

    let visits: Vec<VisitDetailView> = if filter_status == "ALL" {
        let sql = format!(
            "{} WHERE v.requested_date = ?1 ORDER BY v.created_at DESC",
            SELECT_VISIT_DETAIL_SQL
        );
        sqlx::query_as(&sql)
            .bind(&filter_date)
            .fetch_all(&state.db)
            .await?
    } else {
        let sql = format!(
            "{} WHERE v.requested_date = ?1 AND v.current_status = ?2 ORDER BY v.created_at DESC",
            SELECT_VISIT_DETAIL_SQL
        );
        sqlx::query_as(&sql)
            .bind(&filter_date)
            .bind(&filter_status)
            .fetch_all(&state.db)
            .await?
    };

    let santri_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM santri WHERE is_active = 1")
        .fetch_one(&state.db)
        .await?;
    let tahfidz_total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tahfidz_logs")
        .fetch_one(&state.db)
        .await?;
    let exam_total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM exams")
        .fetch_one(&state.db)
        .await?;

    Ok(HtmlTemplate(AdminDashboardTemplate {
        logo_url: PONDOK_LOGO_URL,
        user_name: user.full_name,
        filter_date,
        filter_status,
        visits,
        santri_count,
        tahfidz_total_count,
        exam_total_count,
    }))
}

pub async fn musyrif_board_page(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(query): Query<DormBoardQuery>,
) -> Result<HtmlTemplate<MusyrifBoardTemplate>, AppError> {
    let today_date = Utc::now().format("%Y-%m-%d").to_string();
    let all_dorms: Vec<Dormitory> = sqlx::query_as(
        "SELECT id, name, gender_zone, building_location, created_at
         FROM dormitories ORDER BY id ASC"
    )
    .fetch_all(&state.db)
    .await?;

    let target_dorm_id = match user.role {
        UserRole::Musyrif => user.dormitory_id.clone().ok_or(AppError::Forbidden)?,
        UserRole::Admin => query
            .dorm_id
            .or_else(|| all_dorms.first().map(|d| d.id.clone()))
            .ok_or_else(|| AppError::NotFound("Data asrama belum tersedia".into()))?,
        _ => return Err(AppError::Forbidden),
    };

    let current_dorm = all_dorms
        .iter()
        .find(|d| d.id == target_dorm_id)
        .cloned()
        .ok_or_else(|| AppError::NotFound("Asrama tidak ditemukan".into()))?;

    let dorm_santri: Vec<Santri> = sqlx::query_as(
        "SELECT id, nis, full_name, class_level, dormitory_id, room_name, primary_wali_id,
                total_juz_memorized, current_surah_focus, is_active, created_at
         FROM santri WHERE dormitory_id = ?1 AND is_active = 1 ORDER BY room_name ASC, full_name ASC"
    )
    .bind(&target_dorm_id)
    .fetch_all(&state.db)
    .await?;

    let sql = format!(
        "{} WHERE s.dormitory_id = ?1 AND v.current_status IN ('SANTRI_CALLED', 'SANTRI_CONFIRMED') ORDER BY v.updated_at DESC",
        SELECT_VISIT_DETAIL_SQL
    );
    let dorm_visits: Vec<VisitDetailView> = sqlx::query_as(&sql)
        .bind(&target_dorm_id)
        .fetch_all(&state.db)
        .await?;

    let (called_visits, confirmed_visits): (Vec<_>, Vec<_>) = dorm_visits
        .into_iter()
        .partition(|v| v.current_status == "SANTRI_CALLED");

    let recent_attendance: Vec<AttendanceLogView> = sqlx::query_as(
        "SELECT a.id, a.santri_id, s.full_name AS santri_name, a.attendance_date,
                a.activity_type, a.status, a.note, u.full_name AS recorded_by_name, a.created_at
         FROM attendance_logs a
         JOIN santri s ON s.id = a.santri_id
         JOIN users u ON u.id = a.recorded_by_id
         WHERE s.dormitory_id = ?1
         ORDER BY a.attendance_date DESC, a.created_at DESC LIMIT 20"
    )
    .bind(&target_dorm_id)
    .fetch_all(&state.db)
    .await?;

    let recent_notes: Vec<SantriNoteView> = sqlx::query_as(
        "SELECT n.id, n.santri_id, s.full_name AS santri_name, n.note_date,
                n.category, n.title, n.description, u.full_name AS recorded_by_name, n.created_at
         FROM santri_notes n
         JOIN santri s ON s.id = n.santri_id
         JOIN users u ON u.id = n.recorded_by_id
         WHERE s.dormitory_id = ?1
         ORDER BY n.note_date DESC, n.created_at DESC LIMIT 20"
    )
    .bind(&target_dorm_id)
    .fetch_all(&state.db)
    .await?;

    Ok(HtmlTemplate(MusyrifBoardTemplate {
        logo_url: PONDOK_LOGO_URL,
        user_name: user.full_name,
        today_date,
        is_admin: user.role == UserRole::Admin,
        current_dorm,
        all_dorms,
        dorm_santri,
        called_visits,
        confirmed_visits,
        recent_attendance,
        recent_notes,
    }))
}
