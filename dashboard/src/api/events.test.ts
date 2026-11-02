import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { subscribeToEngineEvents, EngineEvent } from './events';

describe('subscribeToEngineEvents', () => {
  const originalFetch = globalThis.fetch;

  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
  });

  it('receives and parses SSE data events', async () => {
    const eventsReceived: EngineEvent[] = [];

    const stream = new ReadableStream({
      start(controller) {
        controller.enqueue(
          new TextEncoder().encode(
            'data: {"type":"query_completed","latency_ms":3.45,"action":"classify"}\n\n'
          )
        );
        controller.enqueue(
          new TextEncoder().encode(
            'data: {"type":"node_created","id":10001}\n\n'
          )
        );
        controller.close();
      },
    });

    globalThis.fetch = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      body: stream,
    });

    const unsubscribe = subscribeToEngineEvents({
      onEvent: ev => eventsReceived.push(ev),
    });

    // Wait for stream to process
    await new Promise(resolve => setTimeout(resolve, 50));

    expect(eventsReceived).toHaveLength(2);
    expect(eventsReceived[0]).toEqual({
      type: 'query_completed',
      latency_ms: 3.45,
      action: 'classify',
    });
    expect(eventsReceived[1]).toEqual({
      type: 'node_created',
      id: 10001,
    });

    unsubscribe();
  });
});
