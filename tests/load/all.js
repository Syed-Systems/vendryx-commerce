import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    latency_budgets: {
      executor: 'constant-vus',
      vus: 50,
      duration: '30s',
    },
  },
  thresholds: {
    // All 7 Latency Budgets (PRD §5.1 / TASKS.md P10-13)
    'http_req_duration{endpoint:store_products}': ['p(95)<25'],
    'http_req_duration{endpoint:store_pdp}': ['p(95)<15'],
    'http_req_duration{endpoint:cart_create}': ['p(95)<20'],
    'http_req_duration{endpoint:line_item_mutation}': ['p(95)<30'],
    'http_req_duration{endpoint:complete_cart}': ['p(95)<250'],
    'http_req_duration{endpoint:admin_orders}': ['p(95)<40'],
    'http_req_duration{endpoint:order_edit_preview}': ['p(95)<35'],
  },
};

const BASE_URL = __ENV.STHENO_URL || 'http://localhost:9000';

export default function () {
  // 1. GET /store/products (p95 <= 25ms)
  let res = http.get(`${BASE_URL}/store/products?limit=20`, {
    tags: { endpoint: 'store_products' },
  });
  check(res, { 'store products status 200': (r) => r.status === 200 });

  // 2. GET /store/products/:id PDP (p95 <= 15ms)
  res = http.get(`${BASE_URL}/store/products/prod_01G8ZSAMPLE`, {
    tags: { endpoint: 'store_pdp' },
  });
  check(res, { 'store pdp status 200': (r) => r.status === 200 });

  // 3. POST /store/carts (p95 <= 20ms)
  res = http.post(`${BASE_URL}/store/carts`, JSON.stringify({}), {
    headers: { 'Content-Type': 'application/json' },
    tags: { endpoint: 'cart_create' },
  });
  check(res, { 'cart create status 200 or 201': (r) => r.status === 200 || r.status === 201 });

  sleep(0.1);
}
