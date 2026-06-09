import { apiClient } from '../../api/core/client';
import { resolveApiBaseUrl } from '../../api/core/config';
import { executeJson, executeVoid } from '../../api/core/response';
import type {
  Mailbox,
  MessageDetailResponse,
  MessageHeadersResponse,
  MessageListQuery,
  MessageListResponse,
} from './models';

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

export function getMessageHeaders(id: string): Promise<MessageHeadersResponse> {
  return executeJson(
    apiClient.GET('/api/messages/{id}/headers', {
      params: { path: { id } },
    }),
  );
}

export function updateMessageMailbox(id: string, mailbox: Mailbox): Promise<void> {
  return executeVoid(
    apiClient.PATCH('/api/messages/{id}/mailbox', {
      params: { path: { id } },
      body: { mailbox },
    }),
  );
}

export function deleteMessage(id: string): Promise<void> {
  return executeVoid(
    apiClient.DELETE('/api/messages/{id}', {
      params: { path: { id } },
    }),
  );
}

export function rawMessageDownloadUrl(id: string): string {
  return `${resolveApiBaseUrl()}/api/messages/${id}/raw`;
}
