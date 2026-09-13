use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::{error::AppError, models::FacetValue, routes::AppState};

#[derive(Debug, Deserialize)]
pub struct FacetQuery {
    kind: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/facets",
    operation_id = "listFacets",
    params(("kind" = Option<String>, Query, description = "recipient, sender, or domain")),
    responses((status = 200, description = "Non-empty system facets", body = [FacetValue]))
)]
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<FacetQuery>,
) -> Result<Json<Vec<FacetValue>>, AppError> {
    Ok(Json(state.repo.list_facets(query.kind.as_deref()).await?))
}
