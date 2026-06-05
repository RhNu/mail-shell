import { createQuery } from '@tanstack/solid-query';
import type { Accessor } from 'solid-js';
import { getMessageDetail, listMessages } from './api';
import type { MessageListQuery } from './models';

const messagesKeys = {
  all: ['messages'] as const,
  list: (query: MessageListQuery) => [...messagesKeys.all, 'list', query] as const,
  detail: (id: string) => [...messagesKeys.all, 'detail', id] as const,
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
