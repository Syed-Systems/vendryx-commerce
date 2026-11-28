from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if os.environ.get("UPSTREAM_NAME") == "storefront" and self.path in {
            "/__unknown__",
            "/admin/nope",
        }:
            body = b"Not Found"
            self.send_response(404)
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return

        body = json.dumps(
            {
                "upstream": os.environ.get("UPSTREAM_NAME"),
                "host": self.headers.get("Host"),
                "country": self.headers.get("x-country-code"),
                "cloud_country": self.headers.get("CF-IPCountry"),
                "forwarded_host": self.headers.get("X-Forwarded-Host"),
                "tenant_host": self.headers.get("x-stheno-tenant-host"),
                "path": self.path,
            }
        ).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_args):
        pass


ThreadingHTTPServer(("0.0.0.0", 8080), Handler).serve_forever()
