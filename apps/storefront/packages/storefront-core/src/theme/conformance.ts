import type { ThemeContract } from "./contract"

import * as boundTheme from "./index"

/**
 * Compile-time proof that the theme bound in ./index supplies every slot in the
 * contract. A theme missing one fails `npm run typecheck`, not review.
 *
 * Nothing imports this module at runtime, so it adds no bundle weight.
 */
const conformance: ThemeContract = boundTheme

export default conformance
