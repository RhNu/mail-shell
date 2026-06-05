import { createQuery } from '@tanstack/solid-query';
import { checkHealth } from '../api';

const healthKeys = {
  all: ['health'] as const,
  check: () => [...healthKeys.all, 'check'] as const,
};

export function useHealthCheck() {
  return createQuery(() => ({
    queryKey: healthKeys.check(),
    queryFn: () => checkHealth(),
  }));
}
