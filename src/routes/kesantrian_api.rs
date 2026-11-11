use axum::{
    extract::{Form, State},
    http::{header::HeaderName, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::{
    auth::CurrentUser,
    error::AppError,
    services::kesantrian_service::{
        create_santri_note, upsert_attendance_log, CreateSantriNoteInput, RecordAttendanceInput,
    },
    AppState,
};

pub async fn handle_upsert_attendance(
    State(state): State<AppState>,
    user: CurrentUser,
    Form(input): Form<RecordAttendanceInput>,
) -> Result<impl IntoResponse, AppError> {
    let row = upsert_attendance_log(&state, &user, input).await?;
    Ok((
        StatusCode::OK,
        [(HeaderName::from_static("hx-trigger"), "dataChanged")],
        Json(row),
    ))
}

pub async fn handle_create_santri_note(
    State(state): State<AppState>,
    user: CurrentUser,
    Form(input): Form<CreateSantriNoteInput>,
) -> Result<impl IntoResponse, AppError> {
    let row = create_santri_note(&state, &user, input).await?;
    Ok((
        StatusCode::CREATED,
        [(HeaderName::from_static("hx-trigger"), "dataChanged")],
        Json(row),
    ))
}
