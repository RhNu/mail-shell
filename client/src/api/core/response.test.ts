import { describe, expect, it } from 'vitest';
import { executeJson } from './response';
import { HttpResponseError, ResponseParseError } from './errors';

describe('executeJson', () => {
  it('returns parsed data for a successful response', async () => {
    const result = await executeJson(
      Promise.resolve({
        data: { status: 'ok' },
        response: new Response(JSON.stringify({ status: 'ok' }), { status: 200 }),
      }),
    );

    expect(result).toEqual({ status: 'ok' });
  });

  it('throws an http error for a non-success result', async () => {
    const request = executeJson(
      Promise.resolve({
        error: { error: 'message missing' },
        response: new Response(JSON.stringify({ error: 'message missing' }), { status: 404 }),
      }),
    );

    await expect(request).rejects.toBeInstanceOf(HttpResponseError);
  });

  it('throws a parse error when success payload is empty', async () => {
    const request = executeJson(
      Promise.resolve({
        response: new Response(null, { status: 204 }),
      }),
    );

    await expect(request).rejects.toBeInstanceOf(ResponseParseError);
  });
});
