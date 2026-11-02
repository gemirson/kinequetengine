/**
 * HTTP Client for KineContext Engine (KCE) API.
 */

export const API_BASE_URL =
  (import.meta.env.VITE_API_URL as string | undefined)?.replace(/\/+$/, '') ||
  'http://localhost:3000';

export const API_KEY =
  (import.meta.env.VITE_API_KEY as string | undefined) || 'kce-dev-key';

export const TENANT_ID =
  (import.meta.env.VITE_TENANT_ID as string | undefined) || 'tenant-dev';

export class ApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly code?: string
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${API_BASE_URL}${endpoint.startsWith('/') ? endpoint : `/${endpoint}`}`;

  const headers = new Headers(options.headers);
  if (!headers.has('x-api-key')) {
    headers.set('x-api-key', API_KEY);
  }
  if (!headers.has('x-tenant-id')) {
    headers.set('x-tenant-id', TENANT_ID);
  }
  if (options.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(url, {
    ...options,
    headers,
  });

  if (response.status === 204) {
    return undefined as unknown as T;
  }

  const contentType = response.headers.get('content-type') || '';
  const isJson = contentType.includes('application/json');

  if (!response.ok) {
    let errorMessage = `HTTP ${response.status}: ${response.statusText}`;
    let errorCode: string | undefined;

    if (isJson) {
      try {
        const errorJson = await response.json();
        errorMessage = errorJson.message || errorJson.error || errorMessage;
        errorCode = errorJson.error || errorJson.code;
      } catch {
        // fallback to default error message
      }
    } else {
      try {
        const text = await response.text();
        if (text) errorMessage = text;
      } catch {
        // ignore
      }
    }

    throw new ApiError(errorMessage, response.status, errorCode);
  }

  if (isJson) {
    return (await response.json()) as T;
  }

  return (await response.text()) as unknown as T;
}
