import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    admin_list_latency: {
      executor: 'constant-vus',
      vus: 10,
      duration: '5s',
    },
  },
  thresholds: {
    // Admin list latency budget (PRD §5.1 / TASKS.md P17-32: p95 < 50ms)
    'http_req_duration{endpoint:admin_products}': ['p(95)<50'],
    'http_req_duration{endpoint:admin_orders}': ['p(95)<50'],
    'http_req_duration{endpoint:admin_customers}': ['p(95)<50'],
    'http_req_duration{endpoint:admin_inventory}': ['p(95)<50'],
    'http_req_duration{endpoint:admin_promotions}': ['p(95)<50'],
  },
};

const BASE_URL = __ENV.STHENO_URL || 'http://localhost:9000';

export default function () {
  // 1. Admin Products List
  let resProd = http.get(`${BASE_URL}/admin/products?limit=20`, {
    tags: { endpoint: 'admin_products' },
  });
  check(resProd, { 'admin products status 200': (r) => r.status === 200 });

  // 2. Admin Orders List
  let resOrd = http.get(`${BASE_URL}/admin/orders?limit=20`, {
    tags: { endpoint: 'admin_orders' },
  });
  check(resOrd, { 'admin orders status 200': (r) => r.status === 200 });

  // 3. Admin Customers List
  let resCust = http.get(`${BASE_URL}/admin/customers?limit=20`, {
    tags: { endpoint: 'admin_customers' },
  });
  check(resCust, { 'admin customers status 200': (r) => r.status === 200 });

  // 4. Admin Inventory Items List
  let resInv = http.get(`${BASE_URL}/admin/inventory-items?limit=20`, {
    tags: { endpoint: 'admin_inventory' },
  });
  check(resInv, { 'admin inventory status 200': (r) => r.status === 200 });

  // 5. Admin Promotions List
  let resPromo = http.get(`${BASE_URL}/admin/promotions?limit=20`, {
    tags: { endpoint: 'admin_promotions' },
  });
  check(resPromo, { 'admin promotions status 200': (r) => r.status === 200 });

  sleep(0.05);
}
