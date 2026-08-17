#!/usr/bin/env node
// scripts/load-driver.js — minimal constant-rate HTTP load driver.
//
// Used by soak.sh when k6 is not installed. It is deliberately small: open a
// keep-alive agent, fire a fixed number of requests per second across a weighted
// mix of read endpoints, and report real counts. It measures nothing about the
// server process itself — soak.sh samples RSS separately.
//
// Usage: node load-driver.js --url http://127.0.0.1:9000 --rps 500 --duration 60

const http = require("http");

function arg(name, fallback) {
  const i = process.argv.indexOf(`--${name}`);
  return i !== -1 && process.argv[i + 1] ? process.argv[i + 1] : fallback;
}

const base = new URL(arg("url", "http://127.0.0.1:9000"));
const rps = parseInt(arg("rps", "500"), 10);
const durationSec = parseInt(arg("duration", "60"), 10);

// Weighted toward catalog reads, which is what the latency budgets in TRD target.
const paths = [
  "/store/products",
  "/store/products",
  "/store/regions",
  "/store/collections",
  "/health",
];

const agent = new http.Agent({ keepAlive: true, maxSockets: 256 });
let sent = 0;
let ok = 0;
let failed = 0;
let latencyTotalMs = 0;
let maxLatencyMs = 0;

function fire(path) {
  const started = process.hrtime.bigint();
  sent++;
  const req = http.request(
    { hostname: base.hostname, port: base.port, path, method: "GET", agent },
    (res) => {
      res.resume();
      res.on("end", () => {
        const ms = Number(process.hrtime.bigint() - started) / 1e6;
        latencyTotalMs += ms;
        if (ms > maxLatencyMs) maxLatencyMs = ms;
        if (res.statusCode >= 200 && res.statusCode < 400) ok++;
        else failed++;
      });
    }
  );
  req.on("error", () => failed++);
  req.end();
}

let tick = 0;
const timer = setInterval(() => {
  for (let i = 0; i < rps; i++) fire(paths[(tick * rps + i) % paths.length]);
  tick++;
  if (tick >= durationSec) {
    clearInterval(timer);
    // Let in-flight responses land before reporting.
    setTimeout(() => {
      const mean = sent ? (latencyTotalMs / (ok + failed || 1)).toFixed(2) : "0";
      console.log(
        JSON.stringify({
          sent,
          ok,
          failed,
          error_rate: sent ? +(failed / sent).toFixed(6) : 0,
          mean_latency_ms: +mean,
          max_latency_ms: +maxLatencyMs.toFixed(2),
        })
      );
      process.exit(failed > 0 ? 1 : 0);
    }, 5000);
  }
}, 1000);
