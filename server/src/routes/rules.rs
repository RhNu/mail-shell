use axum::{
    Json,
    extract::{Path, State},
};

use crate::{
    error::AppError,
    models::{ClassificationRule, RuleWriteRequest},
    routes::AppState,
};

#[utoipa::path(get, path = "/api/rules", operation_id = "listRules", responses((status = 200, body = [ClassificationRule])))]
pub async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<ClassificationRule>>, AppError> {
    Ok(Json(state.repo.list_rules(false).await?))
}

#[utoipa::path(post, path = "/api/rules", operation_id = "createRule", request_body = RuleWriteRequest, responses((status = 201, body = ClassificationRule)))]
pub async fn create(
    State(state): State<AppState>,
    Json(request): Json<RuleWriteRequest>,
) -> Result<(axum::http::StatusCode, Json<ClassificationRule>), AppError> {
    Ok((
        axum::http::StatusCode::CREATED,
        Json(state.repo.create_rule(&request).await?),
    ))
}

#[utoipa::path(patch, path = "/api/rules/{id}", operation_id = "updateRule", params(("id" = i64, Path)), request_body = RuleWriteRequest, responses((status = 204)))]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<RuleWriteRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    if !state.repo.update_rule(id, &request).await? {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

#[utoipa::path(delete, path = "/api/rules/{id}", operation_id = "deleteRule", params(("id" = i64, Path)), responses((status = 204)))]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<axum::http::StatusCode, AppError> {
    if !state.repo.delete_rule(id).await? {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}
