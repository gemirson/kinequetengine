export const options = {
  stages: [
    { duration: '30s', target: 100 },   // ramp up
    { duration: '1m', target: 1000 },    // sustained load
    { duration: '30s', target: 5000 },   // spike
    { duration: '30s', target: 0 },      // ramp down
  ],
  thresholds: {
    http_req_duration: ['p(99)<10', 'p(95)<5', 'p(50)<2'],
    http_req_failed: ['rate<0.01'],
  },
};
