import { apiClient } from '../../api/core/client';
import { resolveApiBaseUrl } from '../../api/core/config';
import { executeJson, executeVoid } from '../../api/core/response';
import type {
  Mailbox,
  MessageDetailResponse,
  MessageHeadersResponse,
  MessageListQuery,
  MessageListResponse,
  MessageStateUpdateRequest,
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

export function updateMessageState(id: string, state: MessageStateUpdateRequest): Promise<void> {
  return executeVoid(
    apiClient.PATCH('/api/messages/{id}/state', {
      params: { path: { id } },
      body: state,
    }),
  );
}

export function updateMessagesState(
  ids: string[],
  state: MessageStateUpdateRequest,
): Promise<void> {
  return executeVoid(
    apiClient.PATCH('/api/messages/bulk-state', {
      body: { ids, ...state },
    }),
  );
}

export function markAllMessagesRead(query: MessageListQuery = {}): Promise<void> {
  return executeVoid(
    apiClient.PATCH('/api/messages/read-all', {
      params: {
        query: {
          label: query.label,
          mailbox: query.mailbox,
          q: query.q,
          read: query.read,
          starred: query.starred,
          trashed: query.trashed,
        },
      },
    }),
  );
}

export function emptyTrash(): Promise<void> {
  return executeVoid(apiClient.DELETE('/api/trash'));
}

export function rawMessageDownloadUrl(id: string): string {
  return `${resolveApiBaseUrl()}/api/messages/${id}/raw`;
}
