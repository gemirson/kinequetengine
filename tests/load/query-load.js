import http from 'k6/http';
import { check, sleep } from 'k6';

const BASE_URL = __ENV.KCE_URL || 'http://localhost:8080';

export default function () {
  const payload = JSON.stringify({
    query_vector: Array.from({length: 128}, () => Math.random()),
    top_k: 10,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': __ENV.KCE_API_KEY || 'test-key',
      'x-tenant-id': 'load-test',
    },
  };

  const res = http.post(`${BASE_URL}/query`, payload, params);

  check(res, {
    'status is 200': (r) => r.status === 200,
    'latency < 10ms': (r) => r.timings.duration < 10,
    'has pipeline_ms': (r) => JSON.parse(r.body).pipeline_ms !== undefined,
  });

  sleep(0.01);
}
