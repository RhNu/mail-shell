use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    error::AppError,
    models::{Label, LabelWriteRequest, MessageLabelsUpdateRequest},
    routes::AppState,
};

#[utoipa::path(get, path = "/api/labels", operation_id = "listLabels", responses((status = 200, body = [Label])))]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Label>>, AppError> {
    Ok(Json(state.repo.list_labels().await?))
}

#[utoipa::path(post, path = "/api/labels", operation_id = "createLabel", request_body = LabelWriteRequest, responses((status = 201, body = Label)))]
pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<LabelWriteRequest>,
) -> Result<(axum::http::StatusCode, Json<Label>), AppError> {
    Ok((
        axum::http::StatusCode::CREATED,
        Json(state.repo.create_label(&request).await?),
    ))
}

#[utoipa::path(patch, path = "/api/labels/{id}", operation_id = "updateLabel", params(("id" = i64, Path)), request_body = LabelWriteRequest, responses((status = 204)))]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<LabelWriteRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    if !state.repo.update_label(id, &request).await? {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/api/labels/{id}", operation_id = "deleteLabel", params(("id" = i64, Path)), responses((status = 204)))]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    if !state.repo.delete_label(id).await? {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(put, path = "/api/messages/{id}/labels", operation_id = "setMessageLabels", params(("id" = String, Path)), request_body = MessageLabelsUpdateRequest, responses((status = 204)))]
pub async fn set_message_labels(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<MessageLabelsUpdateRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    if !state
        .repo
        .set_message_labels(&id, &request.label_ids)
        .await?
    {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}
