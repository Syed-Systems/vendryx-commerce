import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    catalog_read_latency: {
      executor: 'constant-vus',
      vus: 20,
      duration: '10s',
    },
  },
  thresholds: {
    // Catalog Read Latency Budget (PRD §5.1 / TASKS.md P20-07: p95 < 25ms)
    'http_req_duration{endpoint:products_list}': ['p(95)<25'],
    'http_req_duration{endpoint:product_detail}': ['p(95)<25'],
  },
};

const BASE_URL = __ENV.STHENO_URL || 'http://localhost:9000';

export default function () {
  // 1. GET /store/products
  let resList = http.get(`${BASE_URL}/store/products?limit=20`, {
    tags: { endpoint: 'products_list' },
  });
  check(resList, { 'products list status 200': (r) => r.status === 200 });

  // 2. GET /store/products/:id
  let resDetail = http.get(`${BASE_URL}/store/products/prod_1`, {
    tags: { endpoint: 'product_detail' },
  });
  check(resDetail, { 'product detail status 200 or 404': (r) => r.status === 200 || r.status === 404 });

  sleep(0.05);
}
