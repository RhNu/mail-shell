import { QueryClient } from '@tanstack/solid-query';
import { HttpResponseError } from '../api/core/errors';

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      retry: (failureCount, error: unknown) => {
        if (error instanceof HttpResponseError && error.status >= 400 && error.status < 500) {
          return false;
        }
        return failureCount < 3;
      },
      refetchOnWindowFocus: true,
    },
  },
});
