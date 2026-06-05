use axum::Json;
use utoipa::OpenApi;

use crate::models::{
    AttachmentMeta, ErrorResponse, InboundHeaders, InboundMetadata, InboundResponse, MessageDetail,
    MessageDetailResponse, MessageListResponse, MessageSummary, Tag,
};

#[allow(dead_code)]
#[derive(utoipa::ToSchema)]
pub(crate) struct InboundMultipartRequest {
    #[schema(value_type = String, format = Binary)]
    raw_mime: String,
    metadata: InboundMetadata,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health::handler,
        crate::routes::inbound::handler,
        crate::routes::messages::list,
        crate::routes::messages::detail,
        crate::routes::attachments::download,
        crate::routes::tags::list,
    ),
    components(schemas(
        AttachmentMeta,
        ErrorResponse,
        InboundHeaders,
        InboundMetadata,
        InboundMultipartRequest,
        InboundResponse,
        MessageDetail,
        MessageDetailResponse,
        MessageListResponse,
        MessageSummary,
        Tag,
        crate::routes::health::HealthResponse,
    ))
)]
struct ApiDoc;

pub fn openapi_doc() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub async fn handler() -> Json<utoipa::openapi::OpenApi> {
    Json(openapi_doc())
}
