#!/usr/bin/env node
/**
 * P27-08 — run next with a frontend bound, or several at once.
 *
 *   node scripts/frontend.js matrix dev -p 8001
 *   node scripts/frontend.js --parallel default:8000 matrix:8001
 *
 * The frontend reaches next.config.js as STHENO_FRONTEND; everything after it is
 * passed to next untouched. Each frontend gets its own distDir, so parallel
 * processes do not fight over .next.
 */
const { spawn } = require("child_process")

const run = (frontend, args) =>
  spawn("npx", ["next", ...args], {
    stdio: "inherit",
    shell: true,
    env: { ...process.env, STHENO_FRONTEND: frontend },
  })

const argv = process.argv.slice(2)

if (argv[0] === "--parallel") {
  const children = argv.slice(1).map((spec) => {
    const [frontend, port] = spec.split(":")
    console.log(`\n▸ ${frontend} → http://localhost:${port}`)
    return run(frontend, ["dev", "-p", port])
  })
  const stop = () => children.forEach((c) => c.kill())
  process.on("SIGINT", stop)
  process.on("SIGTERM", stop)
} else {
  const [frontend, ...args] = argv
  run(frontend, args).on("exit", (code) => process.exit(code ?? 0))
}
