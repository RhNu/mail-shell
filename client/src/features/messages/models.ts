import type { components, operations } from '../../api/generated/schema';

export type MessageListQuery = operations['listMessages']['parameters']['query'];
export type MessageListResponse = components['schemas']['MessageListResponse'];
export type MessageSummary = components['schemas']['MessageSummary'];
export type MessageDetail = components['schemas']['MessageDetail'];
export type MessageDetailResponse = components['schemas']['MessageDetailResponse'];
