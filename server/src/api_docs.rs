use axum::Json;
use utoipa::OpenApi;

use crate::models::{
    AttachmentMeta, BulkMessageStateUpdateRequest, ClassificationRule, ErrorResponse, FacetValue,
    HeaderEntry, InboundMetadata, InboundResponse, Label, LabelWriteRequest, Mailbox,
    MailboxUpdateRequest, MessageDetail, MessageDetailResponse, MessageHeadersResponse,
    MessageLabel, MessageLabelsUpdateRequest, MessageListResponse, MessageStateUpdateRequest,
    MessageSummary, RuleActions, RuleCondition, RuleWriteRequest, SavedView, SavedViewWriteRequest,
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
        crate::routes::messages::mark_all_read,
        crate::routes::messages::empty_trash,
        crate::routes::messages::delete,
        crate::routes::messages::raw_download,
        crate::routes::messages::headers,
        crate::routes::attachments::download,
        crate::routes::facets::list,
        crate::routes::labels::list,
        crate::routes::labels::create,
        crate::routes::labels::update,
        crate::routes::labels::delete,
        crate::routes::labels::set_message_labels,
        crate::routes::rules::list,
        crate::routes::rules::create,
        crate::routes::rules::update,
        crate::routes::rules::delete,
        crate::routes::saved_views::list,
        crate::routes::saved_views::create,
        crate::routes::saved_views::update,
        crate::routes::saved_views::delete,
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
        Label,
        MessageLabel,
        LabelWriteRequest,
        MessageLabelsUpdateRequest,
        ClassificationRule,
        RuleCondition,
        RuleActions,
        RuleWriteRequest,
        SavedView,
        SavedViewWriteRequest,
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
