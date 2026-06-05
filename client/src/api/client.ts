import { throwOnError } from './errors';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? '';

export interface RequestOptions extends RequestInit {
  params?: Record<string, string | number | undefined>;
}

function buildUrl(path: string, params?: Record<string, string | number | undefined>): string {
  const url = new URL(path.replace(/^\//, 'api/'), window.location.origin + API_BASE_URL);
  if (params) {
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined) {
        url.searchParams.set(key, String(value));
      }
    });
  }
  return url.toString();
}

export async function apiFetch(path: string, options: RequestOptions = {}): Promise<Response> {
  const { params, ...init } = options;
  const url = buildUrl(path, params);

  const response = await fetch(url, {
    headers: {
      Accept: 'application/json',
      ...(init.body instanceof FormData ? {} : { 'Content-Type': 'application/json' }),
      ...init.headers,
    },
    ...init,
  });

  return throwOnError(response);
}

export async function apiGet<T>(path: string, options?: RequestOptions): Promise<T> {
  const response = await apiFetch(path, { ...options, method: 'GET' });
  return response.json() as Promise<T>;
}

export async function apiPost<T>(path: string, options?: RequestOptions): Promise<T> {
  const response = await apiFetch(path, { ...options, method: 'POST' });
  return response.json() as Promise<T>;
}
