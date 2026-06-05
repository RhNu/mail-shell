export class ApiError extends Error {
  constructor(
    // eslint-disable-next-line no-unused-vars
    public status: number,
    // eslint-disable-next-line no-unused-vars
    public statusText: string,
    message: string,
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export class NotFoundError extends ApiError {
  constructor(message = 'Resource not found') {
    super(404, 'Not Found', message);
    this.name = 'NotFoundError';
  }
}

export class BadRequestError extends ApiError {
  constructor(message = 'Bad request') {
    super(400, 'Bad Request', message);
    this.name = 'BadRequestError';
  }
}

export class ServerError extends ApiError {
  constructor(message = 'Internal server error') {
    super(500, 'Internal Server Error', message);
    this.name = 'ServerError';
  }
}

export function throwOnError(response: Response): Response {
  if (!response.ok) {
    const message = response.statusText;
    switch (response.status) {
      case 400:
        throw new BadRequestError(message);
      case 404:
        throw new NotFoundError(message);
      case 500:
        throw new ServerError(message);
      default:
        throw new ApiError(response.status, response.statusText, message);
    }
  }
  return response;
}
