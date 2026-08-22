import type { FrontendContract } from "./contract"

import * as boundFrontend from "./index"

/**
 * Compile-time proof that the frontend bound in ./index supplies every slot in the
 * contract. A frontend missing one fails `npm run typecheck`, not review.
 *
 * Nothing imports this module at runtime, so it adds no bundle weight.
 */
const conformance: FrontendContract = boundFrontend

export default conformance
