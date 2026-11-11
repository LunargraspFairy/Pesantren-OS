pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod notification;
pub mod routes;
pub mod seed;
pub mod services;
pub mod sse;

use std::{path::PathBuf, sync::Arc};
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;
use tower_http::{
    compression::CompressionLayer,
    services::ServeDir,
    trace::TraceLayer,
};

use crate::{config::AppConfig, notification::NotificationGateway, sse::SseBroadcaster};

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<AppConfig>,
    pub sse: Arc<SseBroadcaster>,
    pub notifier: Arc<dyn NotificationGateway>,
    pub photo_base_dir: Arc<PathBuf>,
}

pub fn build_app_router(state: AppState) -> Router {
    let body_limit = state.config.max_photo_upload_bytes + 65_536;

    Router::new()
        // HTML SSR Views (Modul 08 - 8 Routes)
        .route("/", get(routes::views::root_redirect))
        .route("/login", get(routes::views::login_page))
        .route("/alur", get(routes::views::public_flow_page))
        .route("/alur/:ticket_code", get(routes::views::public_flow_ticket_page))
        .route("/wali", get(routes::views::wali_dashboard_page))
        .route("/asatidz", get(routes::views::asatidz_dashboard_page))
        .route("/admin", get(routes::views::admin_dashboard_page))
        .route("/musyrif/board", get(routes::views::musyrif_board_page))
        // Auth Routes (Modul 02 - 2 Routes)
        .route("/auth/login", post(routes::auth_routes::handle_login))
        .route("/auth/logout", post(routes::auth_routes::handle_logout))
        // Pillar 1: Sambang & Titipan State Machine API (Modul 03 - 6 Routes)
        .route("/api/visits", post(routes::visit_api::handle_create_visit))
        .route("/api/visits/:id/schedule", post(routes::visit_api::handle_schedule_visit))
        .route("/api/visits/:id/wali-confirm", post(routes::visit_api::handle_wali_confirm))
        .route("/api/visits/:id/call-santri", post(routes::visit_api::handle_call_santri))
        .route("/api/visits/:id/musyrif-confirm", post(routes::visit_api::handle_musyrif_confirm))
        .route("/api/visits/:id/cancel", post(routes::visit_api::handle_cancel_visit))
        // Pillar 1: Logbook & Protected Photo API (Modul 05 - 2 Routes)
        .route("/api/visits/:id/logbook", post(routes::logbook_routes::handle_upload_logbook))
        .route("/api/visits/:id/logbook/photo", get(routes::logbook_routes::get_protected_logbook_photo))
        // Pillar 2: Tahfidz & Mutun Tracker API (Modul 06 - 1 Route)
        .route("/api/tahfidz", post(routes::tahfidz_api::handle_record_tahfidz))
        // Pillar 3: Exams, Grades & Rapor API (Modul 07 - 2 Routes)
        .route("/api/exams", post(routes::academic_api::handle_create_exam))
        .route("/api/exams/grades", post(routes::academic_api::handle_upsert_grade))
        // Pillar 4: Attendance & Kesantrian Notes API (Modul 06 - 2 Routes)
        .route("/api/attendance", post(routes::kesantrian_api::handle_upsert_attendance))
        .route("/api/santri-notes", post(routes::kesantrian_api::handle_create_santri_note))
        // Real-Time SSE Streams (Modul 04 - 2 Routes)
        .route("/api/sse/dormitory/:dorm_id", get(routes::sse_routes::sse_dormitory_stream))
        .route("/api/sse/public-flow", get(routes::sse_routes::sse_public_flow_stream))
        // Healthcheck (1 Route)
        .route("/healthz", get(|| async { "OK" }))
        // Static assets (CSS/JS saja, foto logbook TIDAK ada di sini)
        .nest_service("/static", ServeDir::new("./static"))
        .layer(DefaultBodyLimit::max(body_limit))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
