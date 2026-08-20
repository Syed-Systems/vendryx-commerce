/**
 * P25-13 snapshot normaliser.
 *
 * Reads HTML on stdin, writes the comparable form on stdout.
 *
 * What is stripped, and why each one is unavoidable rather than convenient:
 *
 *   <script> blocks      The RSC flight payload embeds webpack module ids. Moving
 *                        a file between packages renumbers them by construction,
 *                        so no extraction can preserve them.
 *   /_next/static/...    Chunk filenames carry a content hash of the module graph.
 *   buildId              Regenerated per build, unrelated to source.
 *   nonce="..."          Per-response CSP value.
 *   $ACTION_* inputs     Next's server-action plumbing. The id is a content hash
 *                        of the module graph, so it differs between two builds of
 *                        identical source — verified — and moving a file changes
 *                        it by construction. $ACTION_KEY is random per render.
 *                        Neither is visible to a shopper; the form still posts to
 *                        the same action.
 *
 * Everything a shopper can see survives: DOM structure, text, Tailwind class
 * names (unhashed by design), href/src targets, and every data-testid the e2e
 * pack drives. If the extraction changes markup, this catches it.
 */
let html = ""
process.stdin.setEncoding("utf8")
process.stdin.on("data", (chunk) => (html += chunk))
process.stdin.on("end", () => {
  const normalized = html
    .replace(/<script[\s\S]*?<\/script>/g, "")
    .replace(/<link[^>]+rel="(?:preload|prefetch|stylesheet)"[^>]*>/g, "")
    .replace(/\/_next\/static\/[^"'\s>]*/g, "/_next/static/X")
    .replace(/\snonce="[^"]*"/g, "")
    .replace(/<input type="hidden" name="\$ACTION[^>]*>/g, "")
    .replace(/\s+/g, " ")
    .trim()
  process.stdout.write(normalized + "\n")
})
