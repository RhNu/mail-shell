use axum::Json;
use utoipa::OpenApi;

use crate::models::{
    AttachmentMeta, ErrorResponse, HeaderEntry, InboundMetadata, InboundResponse, Mailbox,
    MailboxUpdateRequest, MessageDetail, MessageDetailResponse, MessageHeadersResponse,
    MessageListResponse, MessageSummary, Tag,
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
        crate::routes::messages::update_mailbox,
        crate::routes::messages::delete,
        crate::routes::messages::raw_download,
        crate::routes::messages::headers,
        crate::routes::attachments::download,
        crate::routes::tags::list,
    ),
    components(schemas(
        AttachmentMeta,
        ErrorResponse,
        HeaderEntry,
        InboundMetadata,
        InboundMultipartRequest,
        InboundResponse,
        Mailbox,
        MailboxUpdateRequest,
        MessageDetail,
        MessageDetailResponse,
        MessageHeadersResponse,
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
