import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    search_latency: {
      executor: 'constant-vus',
      vus: 20,
      duration: '10s',
    },
  },
  thresholds: {
    // Search Latency Budget (PRD §5.1 / TASKS.md P20-11: p95 < 25ms)
    'http_req_duration{endpoint:search_products}': ['p(95)<25'],
  },
};

const BASE_URL = __ENV.STHENO_URL || 'http://localhost:9000';

export default function () {
  // 1. GET /store/products?q=shirt
  let resSearch = http.get(`${BASE_URL}/store/products?q=shirt&limit=10`, {
    tags: { endpoint: 'search_products' },
  });
  check(resSearch, { 'search status 200': (r) => r.status === 200 });

  sleep(0.05);
}
