use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    error::AppError,
    models::{SavedView, SavedViewWriteRequest},
    routes::AppState,
};

#[utoipa::path(get, path = "/api/saved-views", operation_id = "listSavedViews", responses((status = 200, body = [SavedView])))]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<SavedView>>, AppError> {
    Ok(Json(state.repo.list_saved_views().await?))
}

#[utoipa::path(post, path = "/api/saved-views", operation_id = "createSavedView", request_body = SavedViewWriteRequest, responses((status = 201, body = SavedView)))]
pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<SavedViewWriteRequest>,
) -> Result<(axum::http::StatusCode, Json<SavedView>), AppError> {
    Ok((
        axum::http::StatusCode::CREATED,
        Json(state.repo.create_saved_view(&request).await?),
    ))
}

#[utoipa::path(patch, path = "/api/saved-views/{id}", operation_id = "updateSavedView", params(("id" = i64, Path)), request_body = SavedViewWriteRequest, responses((status = 204)))]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<SavedViewWriteRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    if !state.repo.update_saved_view(id, &request).await? {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/api/saved-views/{id}", operation_id = "deleteSavedView", params(("id" = i64, Path)), responses((status = 204)))]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    if !state.repo.delete_saved_view(id).await? {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}
