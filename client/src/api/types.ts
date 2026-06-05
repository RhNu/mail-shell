export interface HealthResponse {
  status: string;
  classification_model: string;
}

export interface InboundHeaders {
  'message-id': string;
  subject: string;
  date: string;
}

export interface InboundMetadata {
  from: string;
  to: string;
  headers: InboundHeaders;
}

export interface InboundResponse {
  id: string;
}

export interface MessageSummary {
  id: string;
  from_address: string;
  to_address: string;
  subject: string | null;
  date: string | null;
  message_id: string | null;
  created_at: string;
}

export interface MessageDetail {
  id: string;
  from_address: string;
  to_address: string;
  subject: string | null;
  date: string | null;
  message_id: string | null;
  body_text: string | null;
  body_html: string | null;
  created_at: string;
}

export interface AttachmentMeta {
  id: string;
  message_id: string;
  filename: string | null;
  content_type: string | null;
  size: number | null;
}

export interface MessageDetailResponse {
  message: MessageDetail;
  attachments: AttachmentMeta[];
}

export interface Tag {
  id: number;
  kind: string;
  value: string;
  label: string;
  source: string;
  message_count: number | null;
}

export interface Paginated<T> {
  items: T[];
  total: number;
  page: number;
  limit: number;
}

export interface ListMessagesQuery {
  page?: number;
  limit?: number;
  tag?: number;
}
