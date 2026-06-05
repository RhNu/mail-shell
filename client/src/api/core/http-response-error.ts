export class HttpResponseError extends Error {
  public readonly status: number;
  public readonly statusText: string;
  public readonly body: unknown;

  constructor(status: number, statusText: string, body: unknown, options?: ErrorOptions) {
    super(buildHttpErrorMessage(status, statusText, body), options);
    this.name = 'HttpResponseError';
    this.status = status;
    this.statusText = statusText;
    this.body = body;
  }
}

function buildHttpErrorMessage(status: number, statusText: string, body: unknown): string {
  const message =
    typeof body === 'object' && body !== null && 'error' in body && typeof body.error === 'string'
      ? body.error
      : statusText;

  return `HTTP ${status}: ${message || 'Request failed'}`;
}
