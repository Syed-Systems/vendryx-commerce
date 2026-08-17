import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    cart_write_latency: {
      executor: 'constant-vus',
      vus: 20,
      duration: '10s',
    },
  },
  thresholds: {
    // Cart Write Latency Budget (PRD §5.1 / TASKS.md P20-08: p95 < 20ms)
    'http_req_duration{endpoint:cart_create}': ['p(95)<20'],
    'http_req_duration{endpoint:cart_add_line_item}': ['p(95)<20'],
  },
};

const BASE_URL = __ENV.STHENO_URL || 'http://localhost:9000';

export default function () {
  // 1. POST /store/carts
  let res = http.post(
    `${BASE_URL}/store/carts`,
    JSON.stringify({
      region_id: 'reg_us',
      currency_code: 'usd',
      email: 'k6_cart@example.com',
    }),
    {
      headers: { 'Content-Type': 'application/json' },
      tags: { endpoint: 'cart_create' },
    }
  );

  check(res, { 'cart create status 200 or 201': (r) => r.status === 200 || r.status === 201 });

  if (res.status === 200 || res.status === 201) {
    let body = JSON.parse(res.body);
    let cartId = body.cart ? body.cart.id : body.id;

    if (cartId) {
      // 2. POST /store/carts/:id/line-items
      let lineRes = http.post(
        `${BASE_URL}/store/carts/${cartId}/line-items`,
        JSON.stringify({
          variant_id: 'var_1',
          quantity: 1,
        }),
        {
          headers: { 'Content-Type': 'application/json' },
          tags: { endpoint: 'cart_add_line_item' },
        }
      );
      check(lineRes, {
        'line item added or validated': (r) =>
          r.status === 200 || r.status === 201 || r.status === 400 || r.status === 422,
      });
    }
  }

  sleep(0.05);
}
