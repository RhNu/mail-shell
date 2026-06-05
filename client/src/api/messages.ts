import { apiGet } from './client';
import type { ListMessagesQuery, MessageDetailResponse, MessageSummary, Paginated } from './types';

export function listMessages(query: ListMessagesQuery = {}): Promise<Paginated<MessageSummary>> {
  return apiGet('/messages', {
    params: {
      page: query.page,
      limit: query.limit,
      tag: query.tag,
    },
  });
}

export function getMessageDetail(id: string): Promise<MessageDetailResponse> {
  return apiGet(`/messages/${id}`);
}
