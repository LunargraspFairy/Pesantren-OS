use axum::{
    extract::{Form, Path, State},
    http::{header::HeaderName, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::{
    auth::CurrentUser,
    error::AppError,
    models::VisitDetailView,
    services::visit_service::{
        call_santri_to_dormitory, cancel_visit, confirm_santri_by_musyrif,
        confirm_schedule_by_wali, create_visit_request, schedule_visit,
        CancelVisitInput, CreateVisitInput, ScheduleVisitInput,
    },
    AppState,
};

fn htmx_json_response(status: StatusCode, detail: VisitDetailView) -> impl IntoResponse {
    (
        status,
        [(HeaderName::from_static("hx-trigger"), "visitChanged")],
        Json(detail),
    )
}

pub async fn handle_create_visit(
    State(state): State<AppState>,
    user: CurrentUser,
    Form(input): Form<CreateVisitInput>,
) -> Result<impl IntoResponse, AppError> {
    let detail = create_visit_request(&state, &user, input).await?;
    Ok(htmx_json_response(StatusCode::CREATED, detail))
}

pub async fn handle_schedule_visit(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(visit_id): Path<String>,
    Form(input): Form<ScheduleVisitInput>,
) -> Result<impl IntoResponse, AppError> {
    let detail = schedule_visit(&state, &user, &visit_id, input).await?;
    Ok(htmx_json_response(StatusCode::OK, detail))
}

pub async fn handle_wali_confirm(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(visit_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let detail = confirm_schedule_by_wali(&state, &user, &visit_id).await?;
    Ok(htmx_json_response(StatusCode::OK, detail))
}

pub async fn handle_call_santri(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(visit_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let detail = call_santri_to_dormitory(&state, &user, &visit_id).await?;
    Ok(htmx_json_response(StatusCode::OK, detail))
}

pub async fn handle_musyrif_confirm(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(visit_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let detail = confirm_santri_by_musyrif(&state, &user, &visit_id).await?;
    Ok(htmx_json_response(StatusCode::OK, detail))
}

pub async fn handle_cancel_visit(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(visit_id): Path<String>,
    Form(input): Form<CancelVisitInput>,
) -> Result<impl IntoResponse, AppError> {
    let detail = cancel_visit(&state, &user, &visit_id, input).await?;
    Ok(htmx_json_response(StatusCode::OK, detail))
}
