import { createQuery } from '@tanstack/solid-query';
import { listTags } from './api';

const tagsKeys = {
  all: ['tags'] as const,
  list: () => [...tagsKeys.all, 'list'] as const,
};

export function useTagsList() {
  return createQuery(() => ({
    queryKey: tagsKeys.list(),
    queryFn: listTags,
  }));
}
