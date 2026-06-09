import { createMutation, createQuery, useQueryClient } from '@tanstack/solid-query';
import type { Accessor } from 'solid-js';
import {
  deleteMessage,
  getMessageDetail,
  getMessageHeaders,
  listMessages,
  updateMessageMailbox,
} from './api';
import type { Mailbox, MessageListQuery } from './models';

const messagesKeys = {
  all: ['messages'] as const,
  list: (query: MessageListQuery) => [...messagesKeys.all, 'list', query] as const,
  detail: (id: string) => [...messagesKeys.all, 'detail', id] as const,
  headers: (id: string) => [...messagesKeys.all, 'headers', id] as const,
};

type UpdateMessageMailboxVariables = {
  id: string;
  mailbox: Mailbox;
};

type DeleteMessageVariables = {
  id: string;
};

export function useMessagesList(query: Accessor<MessageListQuery>) {
  return createQuery(() => ({
    queryKey: messagesKeys.list(query() ?? {}),
    queryFn: () => listMessages(query()),
  }));
}

export function useMessageDetail(id: Accessor<string>) {
  return createQuery(() => ({
    queryKey: messagesKeys.detail(id()),
    queryFn: () => getMessageDetail(id()),
    enabled: Boolean(id()),
  }));
}

export function useMessageHeaders(id: Accessor<string>, enabled: Accessor<boolean>) {
  return createQuery(() => ({
    queryKey: messagesKeys.headers(id()),
    queryFn: () => getMessageHeaders(id()),
    enabled: Boolean(id()) && enabled(),
  }));
}

export function useUpdateMessageMailbox() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: ({ id, mailbox }: UpdateMessageMailboxVariables) =>
      updateMessageMailbox(id, mailbox),
    onSuccess: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: messagesKeys.all }),
        queryClient.invalidateQueries({ queryKey: ['tags'] }),
      ]),
  }));
}

export function useDeleteMessage() {
  const queryClient = useQueryClient();

  return createMutation(() => ({
    mutationFn: ({ id }: DeleteMessageVariables) => deleteMessage(id),
    onSuccess: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: messagesKeys.all }),
        queryClient.invalidateQueries({ queryKey: ['tags'] }),
      ]),
  }));
}
