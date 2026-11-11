use axum::{
    extract::{Form, State},
    http::{header::HeaderName, StatusCode},
    response::IntoResponse,
    Json,
};

use crate::{
    auth::CurrentUser,
    error::AppError,
    services::academic_service::{
        create_exam_session, upsert_exam_grade, CreateExamInput, UpsertExamGradeInput,
    },
    AppState,
};

pub async fn handle_create_exam(
    State(state): State<AppState>,
    user: CurrentUser,
    Form(input): Form<CreateExamInput>,
) -> Result<impl IntoResponse, AppError> {
    let exam = create_exam_session(&state, &user, input).await?;
    Ok((
        StatusCode::CREATED,
        [(HeaderName::from_static("hx-trigger"), "dataChanged")],
        Json(exam),
    ))
}

pub async fn handle_upsert_grade(
    State(state): State<AppState>,
    user: CurrentUser,
    Form(input): Form<UpsertExamGradeInput>,
) -> Result<impl IntoResponse, AppError> {
    let grade = upsert_exam_grade(&state, &user, input).await?;
    Ok((
        StatusCode::OK,
        [(HeaderName::from_static("hx-trigger"), "dataChanged")],
        Json(grade),
    ))
}
