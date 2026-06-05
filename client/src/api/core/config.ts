const configuredBaseUrl = import.meta.env.VITE_API_BASE_URL?.trim();

export function resolveApiBaseUrl(): string {
  if (!configuredBaseUrl) {
    return window.location.origin;
  }

  return new URL(configuredBaseUrl, window.location.origin).toString().replace(/\/$/u, '');
}
