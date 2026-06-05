export class ResponseParseError extends Error {
  constructor(message = 'Failed to parse API response', options?: ErrorOptions) {
    super(message, options);
    this.name = 'ResponseParseError';
  }
}
