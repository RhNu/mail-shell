import { HttpResponseError, NetworkRequestError, ResponseParseError } from './errors';

type ApiResult<TData, TError = unknown> = {
  data?: TData;
  error?: TError;
  response: Response;
};

export async function executeJson<TData, TError = unknown>(
  request: Promise<ApiResult<TData, TError>>,
): Promise<TData> {
  try {
    const result = await request;
    if (!result.response.ok || result.error !== undefined) {
      throw new HttpResponseError(
        result.response.status,
        result.response.statusText,
        result.error ?? null,
      );
    }
    if (result.data === undefined) {
      throw new ResponseParseError('API returned an empty JSON payload');
    }

    return result.data;
  } catch (error) {
    throw normalizeClientError(error);
  }
}

export async function executeBlob<TError = unknown>(
  request: Promise<ApiResult<unknown, TError>>,
): Promise<Blob> {
  try {
    const result = await request;
    if (!result.response.ok || result.error !== undefined) {
      throw new HttpResponseError(
        result.response.status,
        result.response.statusText,
        result.error ?? null,
      );
    }

    return await result.response.blob();
  } catch (error) {
    throw normalizeClientError(error);
  }
}

function normalizeClientError(error: unknown): Error {
  if (
    error instanceof HttpResponseError ||
    error instanceof NetworkRequestError ||
    error instanceof ResponseParseError
  ) {
    return error;
  }

  if (error instanceof Error) {
    return new NetworkRequestError(error.message, { cause: error });
  }

  return new NetworkRequestError();
}
