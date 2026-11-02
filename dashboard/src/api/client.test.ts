import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { apiFetch, ApiError, API_BASE_URL, API_KEY, TENANT_ID } from './client';

describe('apiFetch', () => {
  const originalFetch = globalThis.fetch;

  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
  });

  it('attaches default headers and returns parsed JSON', async () => {
    const mockData = { status: 'Healthy', version: '6.0.0' };
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      headers: new Headers({ 'content-type': 'application/json' }),
      json: async () => mockData,
    });

    const result = await apiFetch<typeof mockData>('/health');

    expect(globalThis.fetch).toHaveBeenCalledWith(
      `${API_BASE_URL}/health`,
      expect.objectContaining({
        headers: expect.any(Headers),
      })
    );

    const callHeaders = (globalThis.fetch as any).mock.calls[0][1].headers as Headers;
    expect(callHeaders.get('x-api-key')).toBe(API_KEY);
    expect(callHeaders.get('x-tenant-id')).toBe(TENANT_ID);
    expect(result).toEqual(mockData);
  });

  it('handles 204 No Content returning undefined', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 204,
      headers: new Headers(),
    });

    const result = await apiFetch('/nodes/123', { method: 'DELETE' });
    expect(result).toBeUndefined();
  });

  it('throws ApiError with status and code on error response', async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      statusText: 'Not Found',
      headers: new Headers({ 'content-type': 'application/json' }),
      json: async () => ({ error: 'NOT_FOUND', message: 'Node not found' }),
    });

    await expect(apiFetch('/nodes/999')).rejects.toThrow(ApiError);

    try {
      await apiFetch('/nodes/999');
    } catch (err) {
      const apiErr = err as ApiError;
      expect(apiErr.status).toBe(404);
      expect(apiErr.code).toBe('NOT_FOUND');
      expect(apiErr.message).toBe('Node not found');
    }
  });
});
