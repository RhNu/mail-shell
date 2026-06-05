export class NetworkRequestError extends Error {
  constructor(message = 'Network request failed', options?: ErrorOptions) {
    super(message, options);
    this.name = 'NetworkRequestError';
  }
}
