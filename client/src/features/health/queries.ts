import { createQuery } from '@tanstack/solid-query';
import { getHealth } from './api';

const healthKeys = {
  all: ['health'] as const,
  detail: () => [...healthKeys.all, 'detail'] as const,
};

export function useHealthQuery() {
  return createQuery(() => ({
    queryKey: healthKeys.detail(),
    queryFn: getHealth,
  }));
}
