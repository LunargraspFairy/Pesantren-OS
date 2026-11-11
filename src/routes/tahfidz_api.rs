use axum::{
    extract::{Form, State},
    http::{header::HeaderName, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::{
    auth::CurrentUser,
    error::AppError,
    services::tahfidz_service::{record_tahfidz_log, CreateTahfidzInput},
    AppState,
};

pub async fn handle_record_tahfidz(
    State(state): State<AppState>,
    user: CurrentUser,
    Form(input): Form<CreateTahfidzInput>,
) -> Result<impl IntoResponse, AppError> {
    let log = record_tahfidz_log(&state, &user, input).await?;
    Ok((
        StatusCode::CREATED,
        [(HeaderName::from_static("hx-trigger"), "dataChanged")],
        Json(log),
    ))
}
