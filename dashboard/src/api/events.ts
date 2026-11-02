/**
 * SSE Client for /events using fetch + ReadableStream (supports headers like x-api-key).
 */

import { API_BASE_URL, API_KEY, TENANT_ID } from './client';

export type EngineEvent =
  | { type: 'query_completed'; latency_ms: number; action: string }
  | { type: 'node_created'; id: number }
  | { type: 'node_removed'; id: number }
  | { type: 'pheromone_deposited'; from: number; to: number; level: number }
  | { type: 'ais_anomaly'; score: number; blocked: boolean };

export type EventCallback = (event: EngineEvent) => void;

export interface EventStreamOptions {
  onEvent: EventCallback;
  onError?: (err: unknown) => void;
  onOpen?: () => void;
}

/**
 * Connect to SSE endpoint with automatic exponential backoff reconnection.
 * Returns an abort / unsubscribe function.
 */
export function subscribeToEngineEvents(options: EventStreamOptions): () => void {
  let isAborted = false;
  let abortController = new AbortController();
  let retryDelay = 1000;
  const MAX_RETRY_DELAY = 10000;

  async function connect() {
    if (isAborted) return;

    abortController = new AbortController();

    try {
      const url = `${API_BASE_URL}/events`;
      const response = await fetch(url, {
        headers: {
          Accept: 'text/event-stream',
          'x-api-key': API_KEY,
          'x-tenant-id': TENANT_ID,
        },
        signal: abortController.signal,
      });

      if (!response.ok) {
        throw new Error(`SSE connection failed: HTTP ${response.status}`);
      }

      options.onOpen?.();
      retryDelay = 1000; // Reset on success

      if (!response.body) {
        throw new Error('No response body for SSE');
      }

      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let buffer = '';

      while (!isAborted) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split(/\r?\n/);
        buffer = lines.pop() || '';

        for (const line of lines) {
          const trimmed = line.trim();
          if (trimmed.startsWith('data:')) {
            const dataStr = trimmed.slice(5).trim();
            if (dataStr) {
              try {
                const parsed = JSON.parse(dataStr) as EngineEvent;
                options.onEvent(parsed);
              } catch (parseErr) {
                console.warn('Failed to parse SSE payload:', dataStr, parseErr);
              }
            }
          }
        }
      }
    } catch (err) {
      if (!isAborted && (err as Error).name !== 'AbortError') {
        options.onError?.(err);
      }
    } finally {
      if (!isAborted) {
        // Reconnect with exponential backoff
        setTimeout(() => {
          retryDelay = Math.min(retryDelay * 1.5, MAX_RETRY_DELAY);
          connect();
        }, retryDelay);
      }
    }
  }

  connect();

  return () => {
    isAborted = true;
    abortController.abort();
  };
}
