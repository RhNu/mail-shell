import { createQuery } from '@tanstack/solid-query';
import type { Accessor } from 'solid-js';
import { listMessages, getMessageDetail } from '../api';
import type { ListMessagesQuery } from '../api';

const messagesKeys = {
  all: ['messages'] as const,
  list: (query: ListMessagesQuery) => [...messagesKeys.all, 'list', query] as const,
  detail: (id: string) => [...messagesKeys.all, 'detail', id] as const,
};

export function useMessagesList(query: Accessor<ListMessagesQuery>) {
  return createQuery(() => ({
    queryKey: messagesKeys.list(query()),
    queryFn: () => listMessages(query()),
  }));
}

export function useMessageDetail(id: Accessor<string>) {
  return createQuery(() => ({
    queryKey: messagesKeys.detail(id()),
    queryFn: () => getMessageDetail(id()),
    enabled: !!id(),
  }));
}
