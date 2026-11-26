import json
import subprocess
import time
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen


BASE = "http://127.0.0.1:18080"


def request(host, path, headers=None):
    status, body, _ = request_with_headers(host, path, headers)
    return status, body


def request_with_headers(host, path, headers=None):
    values = {"Host": host, **(headers or {})}
    req = Request(BASE + path, headers=values)
    try:
        with urlopen(req, timeout=2) as response:
            return response.status, response.read(), response.headers
    except HTTPError as response:
        return response.code, response.read(), response.headers


for _ in range(30):
    try:
        status, body = request("shop.example.test", "/store/products")
        if status == 200:
            break
    except (URLError, TimeoutError):
        time.sleep(1)
else:
    raise SystemExit("proxy did not become ready")

status, body = request(
    "shop.example.test",
    "/store/products",
    {
        "CF-IPCountry": "BD",
        "x-country-code": "ZZ",
        "X-Forwarded-Host": "attacker.example.test",
        "x-stheno-tenant-host": "victim.example.test",
    },
)
assert status == 200
forwarded = json.loads(body)
assert forwarded["upstream"] == "engine"
assert forwarded["host"] == "shop.example.test"
assert forwarded["country"] == "BD"
assert forwarded["cloud_country"] is None
assert forwarded["forwarded_host"] is None
assert forwarded["tenant_host"] is None

status, body = request(
    "shop.example.test", "/store/products", {"x-country-code": "ZZ"}
)
assert status == 200
assert json.loads(body)["country"] is None

status, body = request("tenant.admin.example.test", "/admin/products")
assert status == 200
admin_api = json.loads(body)
assert admin_api["upstream"] == "engine"
assert admin_api["host"] == "tenant.admin.example.test"

status, body, response_headers = request_with_headers("tenant.admin.example.test", "/")
assert status == 200
assert json.loads(body)["upstream"] == "admin"
assert response_headers.get("Content-Security-Policy") == "script-src 'self'; frame-ancestors 'none'"

for path in ("/auth/customer/providers",):
    status, body = request("shop.example.test", path)
    assert status == 200
    assert json.loads(body)["upstream"] == "engine"

for path in ("/auth/user/providers", "/auth/session"):
    status, body = request("shop.example.test", path)
    assert status == 200
    assert json.loads(body)["upstream"] == "storefront"

status, body = request("tenant.admin.example.test", "/auth/user/providers")
assert status == 200
assert json.loads(body)["upstream"] == "engine"

admin_path = request("shop.example.test", "/admin/nope")
unknown_path = request("shop.example.test", "/__unknown__")
assert admin_path == unknown_path == (404, b"Not Found")

missing_suffix = subprocess.run(
    [
        "docker",
        "compose",
        "-f",
        "deploy/proxy/compose.check.yml",
        "run",
        "--rm",
        "-T",
        "--no-deps",
        "-e",
        "STHENO_ADMIN_DOMAIN_SUFFIX=",
        "proxy",
    ],
    capture_output=True,
    text=True,
    timeout=20,
    check=False,
)
assert missing_suffix.returncode != 0
assert "STHENO_ADMIN_DOMAIN_SUFFIX must be set" in (
    missing_suffix.stdout + missing_suffix.stderr
)
