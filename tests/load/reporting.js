import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    reporting_latency: {
      executor: 'constant-vus',
      vus: 10,
      duration: '5s',
    },
  },
  thresholds: {
    // Analytics Latency Budget: p95 < 50ms for daily rollup aggregated queries
    'http_req_duration{endpoint:revenue_over_time}': ['p(95)<50'],
    'http_req_duration{endpoint:orders_over_time}': ['p(95)<50'],
    'http_req_duration{endpoint:top_products}': ['p(95)<50'],
    'http_req_duration{endpoint:average_order_value}': ['p(95)<50'],
  },
};

const BASE_URL = __ENV.STHENO_URL || 'http://localhost:9000';

export default function () {
  // 1. Revenue over time
  let revRes = http.get(`${BASE_URL}/admin/reports/revenue-over-time?currency=USD`, {
    tags: { endpoint: 'revenue_over_time' },
  });
  check(revRes, { 'revenue report status 200': (r) => r.status === 200 });

  // 2. Orders over time
  let ordRes = http.get(`${BASE_URL}/admin/reports/orders-over-time`, {
    tags: { endpoint: 'orders_over_time' },
  });
  check(ordRes, { 'orders report status 200': (r) => r.status === 200 });

  // 3. Top products
  let topRes = http.get(`${BASE_URL}/admin/reports/top-products?limit=10`, {
    tags: { endpoint: 'top_products' },
  });
  check(topRes, { 'top products status 200': (r) => r.status === 200 });

  // 4. Average order value
  let aovRes = http.get(`${BASE_URL}/admin/reports/average-order-value?currency=USD`, {
    tags: { endpoint: 'average_order_value' },
  });
  check(aovRes, { 'aov status 200': (r) => r.status === 200 });

  sleep(0.05);
}
