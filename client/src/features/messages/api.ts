import { apiClient } from '../../api/core/client';
import { executeJson } from '../../api/core/response';
import type { MessageDetailResponse, MessageListQuery, MessageListResponse } from './models';

export function listMessages(query: MessageListQuery = {}): Promise<MessageListResponse> {
  return executeJson(
    apiClient.GET('/api/messages', {
      params: query ? { query } : undefined,
    }),
  );
}

export function getMessageDetail(id: string): Promise<MessageDetailResponse> {
  return executeJson(
    apiClient.GET('/api/messages/{id}', {
      params: { path: { id } },
    }),
  );
}
