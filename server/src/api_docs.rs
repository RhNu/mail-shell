use axum::Json;
use utoipa::OpenApi;

use crate::models::{
    AttachmentMeta, BulkMessageStateUpdateRequest, ErrorResponse, FacetValue, HeaderEntry,
    InboundMetadata, InboundResponse, Mailbox, MailboxUpdateRequest, MessageDetail,
    MessageDetailResponse, MessageHeadersResponse, MessageListResponse, MessageStateUpdateRequest,
    MessageSummary, Tag,
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
        crate::routes::messages::update_state,
        crate::routes::messages::update_bulk_state,
        crate::routes::messages::empty_trash,
        crate::routes::messages::delete,
        crate::routes::messages::raw_download,
        crate::routes::messages::headers,
        crate::routes::attachments::download,
        crate::routes::tags::list,
        crate::routes::facets::list,
    ),
    components(schemas(
        AttachmentMeta,
        BulkMessageStateUpdateRequest,
        ErrorResponse,
        HeaderEntry,
        FacetValue,
        InboundMetadata,
        InboundMultipartRequest,
        InboundResponse,
        Mailbox,
        MailboxUpdateRequest,
        MessageStateUpdateRequest,
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
