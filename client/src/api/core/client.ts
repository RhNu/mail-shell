import createClient, { type Middleware } from 'openapi-fetch';
import type { paths } from '../generated/schema';
import { resolveApiBaseUrl } from './config';

const acceptHeaderMiddleware: Middleware = {
  onRequest({ request }) {
    if (!request.headers.has('Accept')) {
      request.headers.set('Accept', 'application/json, application/octet-stream;q=0.9, */*;q=0.1');
    }

    return request;
  },
};

export const apiClient = createClient<paths>({
  baseUrl: resolveApiBaseUrl(),
});

apiClient.use(acceptHeaderMiddleware);
